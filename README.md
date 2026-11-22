# Harogic WebSA

**Browser spectrum analysis and SDR reception for HAROGIC SAN-45 / SAN-60 / SAN-90.**

[简体中文](README.zh-CN.md) · [Documentation](docs/en/INDEX.md) · [Build and contribute](docs/en/DEVELOPMENT.md)

![WebSA spectrum analyzer](screenshots/main_dark.png)

WebSA combines a native Rust acquisition/DSP service, a C++ adapter for the HAROGIC SDK,
and a TypeScript browser interface. Python runs the crash monitor, offline tools and tests.

Measurements report their bandwidth, method and validity; see [SNR definitions and validation](docs/en/reference/SNR.md).

## What it does

- Swept and real-time spectrum, four traces, density/waterfall, markers, channel power,
  occupied bandwidth, ACPR, harmonic and phase-noise measurements.
- Up to sixteen independently tuned receivers in one capture, local audio mixing with
  stereo routing and overall speaker EQ,
  floating channel/decoder windows and sixteen shared recording task slots.
- AM/DSB/USB/LSB/CW/NFM/WFM/PM, FM stereo/RDS, noise reduction, FT8, DRM/DRM+, SSTV
  and radiofax; optional external decoder providers extend the mode catalog.
- IQ, audio and decoded-result recording, saved-file playback/export, receiver presets,
  English/Chinese UI, twelve appearance themes and shared floating-window opacity.
- In-band signal discovery, spectrum/waterfall region identification, temporary listening
  and adding results to saved channels. See [Scan and identify](docs/en/reference/SIGNAL_SCAN.md).
- Exclusive SoapySDRServer handoff for external SDR clients.

Mode availability depends on installed providers and capture geometry. Sustained sixteen-channel
RF capacity remains unqualified; synthetic validation is not a hardware performance guarantee.
See [Known limitations](docs/en/OPERATIONS.md).

## Feature screenshots

| Real-time spectrum and waterfall | Measurements and markers |
|---|---|
| ![Real-time spectrum and waterfall](screenshots/waterfall_rta.png) | ![Amplitude measurements and peak markers](screenshots/measure_amplitude.png) |
| Multichannel reception and local mixing | Signal discovery and identification |
| ![Channel overview](screenshots/sdr_channels.png) | ![In-band scan and identify](screenshots/sdr_scan.png) |
| Recordings, playback and export | Digital decoding |
| ![Recording library](screenshots/sdr_recordings.png) | ![FT8 decoder](screenshots/sdr_ft8.png) |
| Light theme | |
| ![Spectrum analyzer in the light theme](screenshots/main_light.png) | |

The decoder and recording screenshots use synthetic/replayed IQ. Other screenshots use a SAN-90.

## Try it

### Linux

After cloning, prepare the Linux build prerequisites and run:

```bash
make setup                     # build frontend/service and install all external decoders
make run                       # hardware; use WEBSA_FAKE=1 make run without a device
make decoders-status           # inspect actual decoder availability
```

Decoders live together in `~/.local/share/websa/decoders` and are discovered automatically;
no shell files need sourcing. Setup reuses installed decoders; the first GPS source build is
large. See [First-time setup](docs/en/GETTING_STARTED.md#first-time-setup) for prerequisites,
SDK setup and updates.

The checkout includes a **Linux x86_64/glibc** runtime. The checked-in bundle is v1.12.5;
`make build` produces the current service and frontend. It currently requires glibc 2.44,
GLIBCXX 3.4.30, CXXABI 1.3.15 and `libfdk-aac.so.2`; rebuild if your system cannot run it.

```bash
./run.sh --fake                 # synthetic input, no analyzer required
./status.sh
./stop.sh
```

Open **http://127.0.0.1:8080**. For real hardware, install the separately supplied HAROGIC
SDK under `/opt/htraapi` and run `./run.sh`. The proprietary SDK is not bundled.
[Getting started](docs/en/GETTING_STARTED.md) covers prerequisites, instances and updates.

### Windows x64 and PX Z handhelds

**PXE-90 Z, PXE-200 Z and PXN-400Z** run Windows 11 x64. Build on a Windows x64 development PC or on the instrument after installing the build tools. You can copy the generated application directory to the instrument. [HAROGIC PX Z product page](https://www.harogic.com/product/pxe-z-edition-handheld-spectrum-analyzer/)

The current WebSA adapter opens instruments through the HTRA **USB** interface. Use the handheld's local Windows session and confirm that the SDK can open its analyzer through USB. An analyzer that the SDK exposes only through Ethernet requires an Ethernet adapter in WebSA. The browser's LAN connection to WebSA is independent of the analyzer connection.

Install Visual Studio 2022 Build Tools with **Desktop development with C++** and a Windows SDK, plus CMake, Git, Rust 1.95 or newer, Node.js 22 and Python 3. Extract the [HAROGIC Spectrum Analyzer Windows x64 SDK 0.55.100](https://www.harogic.com/windows-sdk/). `-SdkDirectory` points to the inner `x64\htra_api` directory containing `htra_api.h`, `htra_api_pnm.h`, `htra_api.lib` and `htra_api.dll`.

Run from the repository root in PowerShell. Adjust `$sdkDir` if you extracted the SDK elsewhere.

```powershell
rustup target add x86_64-pc-windows-msvc
$fdkSource = Join-Path $HOME 'fdk-aac'
$sdkDir = Join-Path $HOME 'Downloads\SA_Windows_API\x64\htra_api'
git clone --depth 1 --branch v2.0.3 https://github.com/mstorsjo/fdk-aac.git $fdkSource
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\build-windows.ps1 -FdkSource $fdkSource -SdkDirectory $sdkDir
```

The build writes `websad.exe`, the frontend, `htra_api\websa_san90_sdk_session.dll` and `Start-WebSA.bat` to `windows-build\app`. On the first hardware run, the following command copies the separately supplied SDK DLLs and `CalFile` into the local build and starts WebSA. Use the calibration files for the instrument's UID and close SAStudio4 before running it.

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\run-windows.ps1 -SdkDirectory $sdkDir
```

Open [http://127.0.0.1:8080/](http://127.0.0.1:8080/) on the instrument. Test swept spectrum (STD) before switching to SDR. For later runs, stop the current instance with Ctrl+C and double-click `Start-WebSA.bat` in the application directory. The BAT listens on `0.0.0.0:8080` for access from a trusted LAN. To build and run with simulated data, omit `-SdkDirectory` from the build and run `.\run-windows.ps1 -Fake` from the checkout.

Windows swept spectrum and SDR were confirmed on an N400 R2V2. On a PXN-400 Z, the Windows Soapy SRV release streamed IQ to a Mac over Wi-Fi at about 976.6 kS/s. The SRV panel starts and stops SoapySDRServer and hands the analyzer back to WebSA when returning to Spectrum. The 9.4 GHz and 19.5 GHz SDR tuning and readback tests used a simulated SDK. See the [Windows guide (Chinese)](docs/zh-CN/WINDOWS.md) for deployment, calibration and SRV testing.

For a prebuilt Windows release, extract the application ZIP and copy the separately supplied Windows x64 HTRA SDK runtime DLLs and the unit's calibration files into its `htra_api` directory. The release includes the WebSA SDK shim and the managed Soapy executables and driver.

### Find WebSA on your LAN

Copy [find-websa.py](tools/find-websa.py) to a computer on the same LAN as the analyzer. The script requires Python 3.8 or later and uses the standard library. Run it from the directory containing the script.

```bash
# Linux / macOS
python3 find-websa.py
```

```powershell
# Windows
py -3 find-websa.py
```

Open a URL marked `WebSA:` in your browser. The script retries once after a brief delay if the host or network is unreachable.

If you know the analyzer's address, add `--host 192.168.0.236` to check only that address and see the reason if the check fails. To display only the machine's own IPv4 addresses, add `--ips`. If you have several network interfaces, add `--subnet 192.168.0.0/24` to select a directly connected subnet. Add `--port` followed by the actual port if your server uses a port other than `8080`.

## Build and test

```bash
make native                    # Rust service, one build job
make frontend                  # browser assets
make sdk-shim                  # hardware adapter; vendor SDK required
./test.sh                      # native, Python and frontend tests
```

Run heavyweight commands separately to limit memory use. Restart the intended service after a
native rebuild; an existing process keeps its old executable. Full prerequisites, fixture/browser
checks and packaging are in [Development](docs/en/DEVELOPMENT.md).

## Find your way

| Need | Start here |
|---|---|
| Tune, listen, decode and record | [User guide](docs/en/USER_GUIDE.md) |
| Troubleshoot a device or service | [Operations](docs/en/OPERATIONS.md) |
| Understand the implementation | [Architecture](docs/en/ARCHITECTURE.md) |
| Integrate another client | [API](docs/en/reference/API.md), [receivers](docs/en/reference/RECEIVERS.md), [recordings](docs/en/reference/RECORDINGS.md) |
| Discover signals and identify selected bands | [Scan and identify](docs/en/reference/SIGNAL_SCAN.md) |
| Install optional providers | [Tools](tools/README.md) |

`backend/` owns the service and hardware boundary; `dsp/` owns shared signal-processing
kernels; `frontend/` owns browser interaction; `contracts/` holds shared limits;
`tests/` and `tools/` hold regression fixtures and offline tooling.

## License and attribution

Project-owned code that we can authorize uses [AGPL-3.0-only](LICENSE) with a
[section 7 linking exception](LICENSE-EXCEPTION) for **HAROGIC SDK and FDK AAC only**
(`LicenseRef-Harogic-WebSA`). Project code and modifications retain their applicable
Corresponding Source obligations, including the modified network version's source offer.
The UI links to the license texts and a versioned local source snapshot.

Third-party code, libraries, models and independent decoders retain their original terms;
see [scope and earlier MIT grants](LICENSING.md), [third-party notices](THIRD_PARTY_NOTICES.md)
and [review evidence / unresolved rights](docs/en/LICENSING.md). Earlier MIT releases remain
available under their original [MIT grant](LICENSES/MIT-legacy.txt). The proprietary HAROGIC
SDK must still be obtained separately; the exception supplies no vendor redistribution rights.

Author: [好奇牛马 (Bilibili)](https://space.bilibili.com/28447213).
