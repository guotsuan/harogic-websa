#!/usr/bin/env python3
"""List local IPv4 addresses and find WebSA on directly connected networks."""

import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import errno
import http.client
import ipaddress
import json
import re
import subprocess
import sys
import time


UNREACHABLE_ERRORS = {
    errno.EHOSTUNREACH,
    errno.ENETUNREACH,
    getattr(errno, "WSAEHOSTUNREACH", errno.EHOSTUNREACH),
    getattr(errno, "WSAENETUNREACH", errno.ENETUNREACH),
}


def command_output(command):
    try:
        return subprocess.check_output(command, stderr=subprocess.PIPE, encoding="utf-8", timeout=10)
    except (OSError, subprocess.SubprocessError, UnicodeError) as error:
        raise RuntimeError(f"Could not list IPv4 interfaces with {command[0]}: {error}") from error


def interface_addresses():
    if sys.platform == "win32":
        command = (
            "[Console]::OutputEncoding = [Text.Encoding]::UTF8; "
            "$ErrorActionPreference = 'Stop'; "
            "Get-NetIPAddress -AddressFamily IPv4 | "
            "Where-Object { $_.AddressState -in 'Preferred','Deprecated' } | "
            "Select-Object InterfaceAlias,IPAddress,PrefixLength | ConvertTo-Json -Compress"
        )
        rows = json.loads(command_output(["powershell.exe", "-NoProfile", "-Command", command]) or "[]")
        if isinstance(rows, dict):
            rows = [rows]
        for row in rows:
            yield row["InterfaceAlias"], f"{row['IPAddress']}/{row['PrefixLength']}"
    elif sys.platform.startswith("linux"):
        for line in command_output(["ip", "-o", "-4", "addr", "show", "up"]).splitlines():
            _, name, _, address, *_ = line.split()
            yield name, address
    elif sys.platform == "darwin":
        for line in command_output(["/sbin/ifconfig", "-u"]).splitlines():
            if not line[0:1].isspace():
                name = line.split(":", 1)[0]
            match = re.match(r"\s+inet (\d+\.\d+\.\d+\.\d+) netmask (0x[0-9a-fA-F]+)", line)
            if match:
                mask = ipaddress.IPv4Address(int(match[2], 16))
                yield name, f"{match[1]}/{mask}"
    else:
        raise RuntimeError(f"Unsupported platform: {sys.platform}")


def local_interfaces():
    for name, address in interface_addresses():
        interface = ipaddress.IPv4Interface(address)
        if not (interface.ip.is_loopback or interface.ip.is_unspecified):
            yield name, interface


def probe(address, port, timeout):
    connection = http.client.HTTPConnection(str(address), port, timeout=timeout)
    try:
        connection.request("GET", "/api/state", headers={"Connection": "close"})
        response = connection.getresponse()
        if response.status != 200 or response.headers.get_content_type() != "application/json":
            return None
        state = json.loads(response.read(256 * 1024))
    except (http.client.HTTPException, ValueError):
        return None
    finally:
        connection.close()

    if (isinstance(state, dict) and state.get("mode") in ("std", "rta", "sdr", "server")
            and isinstance(state.get("connected"), bool) and isinstance(state.get("caps"), dict)
            and isinstance(state.get("swp_actual"), dict)):
        return state
    return None


def probe_with_retry(address, port, timeout):
    try:
        return probe(address, port, timeout)
    except OSError as error:
        if error.errno not in UNREACHABLE_ERRORS and getattr(error, "winerror", None) not in UNREACHABLE_ERRORS:
            raise
    time.sleep(0.3)
    return probe(address, port, timeout)


def lan_targets(parser, interfaces, subnet):
    if subnet is not None:
        if not any(subnet.subnet_of(interface.network) for _, interface in interfaces):
            parser.error("--subnet must be within a directly connected network")
        if subnet.prefixlen < 22:
            parser.error("select a /22 or smaller subnet")
        networks = {subnet}
    else:
        networks = {
            interface.network if interface.network.prefixlen >= 22
            else ipaddress.IPv4Network(f"{interface.ip}/24", strict=False)
            for _, interface in interfaces if interface.network.prefixlen < 32
        }

    print("Searching: " + ", ".join(map(str, sorted(networks))) if networks else "No LAN subnets to search.", flush=True)
    addresses = {address for network in networks for address in network.hosts()}
    if subnet is None:
        addresses.update(interface.ip for _, interface in interfaces)
        addresses.add(ipaddress.IPv4Address("127.0.0.1"))
    return addresses


def report_services(addresses, port, timeout):
    found = False
    denied = False
    single_host = len(addresses) == 1
    with ThreadPoolExecutor(max_workers=32) as workers:
        pending = {workers.submit(probe_with_retry, address, port, timeout): address for address in addresses}
        for result in as_completed(pending):
            address = pending[result]
            url = f"http://{address}:{port}/"
            try:
                state = result.result()
            except OSError as error:
                denied |= isinstance(error, PermissionError)
                if single_host:
                    print(f"{url} connection failed: {error}", file=sys.stderr)
                continue
            if state is not None:
                connected = "yes" if state["connected"] else "no"
                print(f"WebSA: {url}  mode={state['mode']}  device connected={connected}", flush=True)
                found = True
            elif single_host:
                print(f"{url} did not return a valid WebSA status response.", file=sys.stderr)
    if denied:
        print("The system denied one or more connections. Run the script in a terminal with LAN access.", file=sys.stderr)
        if not found:
            return 2
    if not found and not single_host:
        print("No WebSA response. Check the server, port, firewall, and Wi-Fi client isolation.")
    return 0 if found else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--ips", action="store_true", help="list local addresses without probing")
    mode.add_argument("--subnet", type=lambda value: ipaddress.IPv4Network(value, strict=False),
                      help="search only this directly connected IPv4 subnet (CIDR)")
    mode.add_argument("--host", type=ipaddress.IPv4Address, help="check a known WebSA IPv4 address")
    parser.add_argument("--port", type=int, default=8080, help="WebSA HTTP port (default: 8080)")
    args = parser.parse_args()
    if not 1 <= args.port <= 65535:
        parser.error("--port must be between 1 and 65535")

    if args.host is not None:
        return report_services({args.host}, args.port, timeout=2.0)

    try:
        interfaces = list(local_interfaces())
    except (RuntimeError, ValueError) as error:
        print(error, file=sys.stderr)
        return 2

    print("Local IPv4 addresses (use these URLs if this machine is running WebSA):")
    for name, interface in interfaces:
        print(f"  {name}: {interface}  http://{interface.ip}:{args.port}/")
    if not interfaces:
        print("  No active IPv4 interfaces found.")
    if args.ips:
        return 0

    return report_services(lan_targets(parser, interfaces, args.subnet), args.port, timeout=0.7)


if __name__ == "__main__":
    sys.exit(main())
