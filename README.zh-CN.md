# Harogic WebSA

**面向 HAROGIC SAN-45／SAN-60／SAN-90 的浏览器频谱分析与 SDR 接收工具。**

[English](README.md) · [文档目录](docs/zh-CN/INDEX.md) · [构建与贡献](docs/zh-CN/DEVELOPMENT.md)

![WebSA 频谱分析仪](screenshots/main_zh.png)

WebSA 由原生 Rust 采集／DSP 服务、连接 HAROGIC SDK 的 C++ 适配器及 TypeScript 浏览器界面组成。
Python 用于崩溃监护、离线工具和测试。

测量报告带宽、方法和有效状态，详见[信噪比定义与验证](docs/zh-CN/reference/SNR.md)。

## 主要功能

- 扫频及实时频谱、四条迹线、密度／瀑布、标记、信道功率、占用带宽、ACPR、谐波和相噪测量。
- 一个采集窗口内最多十六个独立调谐信道、本地混听、声像及整体监听 EQ、信道／解码悬浮窗，以及共享的十六个录制任务名额。
- AM／DSB／USB／LSB／CW／NFM／WFM／PM、FM 立体声／RDS、降噪、FT8、DRM／DRM+、
  SSTV 和无线电传真；可选外部 provider 扩展解码目录。
- IQ、音频及解码结果录制，文件试听／导出，信道预设，中英文界面、十二种外观主题及统一浮窗透明度。
- 带宽内信号发现、频谱／瀑布选区识别、临时试听及将结果加入已保存信道，详见[扫描与识别](docs/zh-CN/reference/SIGNAL_SCAN.md)。
- 向外部 SDR 客户端提供独占 SoapySDRServer 接管。

模式可用性取决于已安装 provider 和采集几何。十六路持续 RF 容量仍待验收；合成验证不构成硬件
性能保证。详见[已知限制](docs/zh-CN/OPERATIONS.md)。

## 功能截图

| 实时频谱与瀑布 | 测量与标记 |
|---|---|
| ![实时频谱与瀑布](screenshots/waterfall_rta_zh.png) | ![幅度测量与峰值标记](screenshots/measure_amplitude_zh.png) |
| 多信道接收与本地混听 | 信号发现与识别 |
| ![信道总览](screenshots/sdr_channels_zh.png) | ![带宽内扫描与识别](screenshots/sdr_scan_zh.png) |
| 录制、试听与导出 | 数字解码 |
| ![录制库](screenshots/sdr_recordings_zh.png) | ![FT8 解码器](screenshots/sdr_ft8_zh.png) |
| 浅色主题 | |
| ![浅色主题频谱分析仪](screenshots/main_light_zh.png) | |

解码和录制截图使用合成／回放 IQ，其余截图使用 SAN-90。

## 试用

### Linux

首次 clone 后，准备 Linux 构建依赖并执行：

```bash
make setup                     # 构建前后端、安装全部外部解码器
make run                       # 启动硬件服务；无硬件用 WEBSA_FAKE=1 make run
make decoders-status           # 检查实际加载情况
```

解码器统一安装到 `~/.local/share/websa/decoders`，启动时自动发现，无需手动 `source`。
`make setup` 复用已安装的解码器；首次 GPS 源码构建较慢。依赖检查、SDK 和更新说明见
[快速开始](docs/zh-CN/GETTING_STARTED.md#首次安装)。

仓库包含 **Linux x86_64/glibc** 运行包。已提交捆绑版本为 v1.12.5，
`make build` 构建当前服务和前端。运行包要求 glibc 2.44、GLIBCXX 3.4.30、
CXXABI 1.3.15 和 `libfdk-aac.so.2`；系统不兼容时请源码构建。

```bash
./run.sh --fake                 # 合成输入，无需分析仪
./status.sh
./stop.sh
```

打开 **http://127.0.0.1:8080**。真实硬件需将单独取得的 HAROGIC SDK 安装于 `/opt/htraapi`，
然后运行 `./run.sh`。仓库不包含专有 SDK。[快速开始](docs/zh-CN/GETTING_STARTED.md)
说明依赖、独立实例及更新方式。

### Windows x64 与 PX Z 手持式频谱仪

**PXE-90 Z、PXE-200 Z 和 PXN-400Z** 搭载 Windows 11 x64。可以在另一台 Windows x64 开发电脑编译，再把生成的应用目录复制到仪器，也可以在仪器安装构建工具后本机编译。 [HAROGIC PX Z 产品页](https://www.harogic.cn/product/pxe-z-edition/)

当前 WebSA 通过 HTRA 的 **USB 接口**打开设备。在手持机本机运行前，请确认厂商 SDK 能通过 USB 打开分析仪。如果 SDK 只通过以太网暴露分析仪，WebSA 需要适配以太网采集。浏览器通过局域网访问 WebSA 与仪器采集接口是两回事。

安装 Visual Studio 2022 Build Tools，勾选 **使用 C++ 的桌面开发**并安装 Windows SDK。还需要 CMake、Git、Rust 1.95 或更新版本、Node.js 22 和 Python 3。解压 [HAROGIC 频谱分析仪 Windows x64 SDK 0.55.100](https://www.harogic.com/windows-sdk/)。`-SdkDirectory` 指向包含 `htra_api.h`、`htra_api_pnm.h`、`htra_api.lib` 和 `htra_api.dll` 的 `x64\htra_api` 目录。

从仓库根目录打开 PowerShell，按实际解压位置修改 `$sdkDir`。

```powershell
rustup target add x86_64-pc-windows-msvc
$fdkSource = Join-Path $HOME 'fdk-aac'
$sdkDir = Join-Path $HOME 'Downloads\SA_Windows_API\x64\htra_api'
git clone --depth 1 --branch v2.0.3 https://github.com/mstorsjo/fdk-aac.git $fdkSource
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\build-windows.ps1 -FdkSource $fdkSource -SdkDirectory $sdkDir
```

构建产物位于 `windows-build\app`，包括 `websad.exe`、网页前端、`htra_api\websa_san90_sdk_session.dll` 和 `Start-WebSA.bat`。首次实机运行用下面的命令把厂商 SDK 运行库和 `CalFile` 复制到本地构建目录并启动 WebSA。校准文件须与仪器 UID 匹配，启动前请退出 SAStudio4。

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\run-windows.ps1 -SdkDirectory $sdkDir
```

在仪器上打开 [http://127.0.0.1:8080/](http://127.0.0.1:8080/)，先试标准扫频（STD），再切换到 SDR。以后按 Ctrl+C 停止旧实例后，可双击应用目录里的 `Start-WebSA.bat` 启动。BAT 监听 `0.0.0.0:8080`，供可信局域网访问。只试模拟模式时，构建可省略 `-SdkDirectory`，然后在仓库根目录运行 `.\run-windows.ps1 -Fake`。

Windows 扫频和 SDR 已在一台 N400 R2V2 上跑通。PXN-400 Z 已通过 Wi-Fi 向 Mac 传输约 976.6 kS/s 的 Soapy IQ。切到 SRV 后可从网页启停 SoapySDRServer，返回频谱时由 WebSA 重新接管仪器。9.4 GHz 和 19.5 GHz 的 SDR 调谐与回读使用模拟 SDK 验证。部署、校准及 SRV 测试步骤见 [Windows 运行指南](docs/zh-CN/WINDOWS.md)。

使用预编译 Windows 发布包时，将另行取得的 Windows x64 HTRA SDK 运行库与仪器校准文件复制到应用的 `htra_api` 目录。发布包已包含 WebSA 的 SDK 适配 DLL、受管 Soapy 服务和驱动。

### 在局域网查找 WebSA

把 [find-websa.py](tools/find-websa.py) 复制到与频谱仪连在同一局域网的电脑上。脚本使用 Python 3.8 或更新版本，无需安装 Python 扩展。在脚本所在目录运行：

```bash
# Linux / macOS
python3 find-websa.py
```

```powershell
# Windows
py -3 find-websa.py
```

脚本列出本机 IPv4 地址，并确认同网段哪些主机在 `8080` 端口响应 WebSA。用浏览器打开输出中以 `WebSA:` 开头的 URL。遇到 `No route to host` 等路由不可达错误时，脚本会短暂等待后重试一次。

已知仪器 IP 时，给命令加 `--host 192.168.0.236` 即可单独检查这个地址。检查失败时会显示具体连接错误，或提示收到的响应无法识别为 WebSA。只想查看本机地址时加 `--ips`。有多个网卡时，可以加 `--subnet 192.168.0.0/24` 指定当前连接的网段。服务改了端口时，加 `--port` 和实际端口号。

## 构建与测试

```bash
make native                    # Rust 服务，单构建任务
make frontend                  # 浏览器资源
make sdk-shim                  # 硬件适配器，需要厂商 SDK
./test.sh                      # 原生、Python 和前端测试
```

重任务分别执行，控制内存占用。重建原生服务后须重启目标实例，旧进程不会自动切换可执行文件。
完整依赖、模拟／浏览器检查和打包方法见[开发指南](docs/zh-CN/DEVELOPMENT.md)。

## 查找资料

| 任务 | 入口 |
|---|---|
| 调谐、监听、解码和录制 | [用户指南](docs/zh-CN/USER_GUIDE.md) |
| 排查设备或服务问题 | [运行与排障](docs/zh-CN/OPERATIONS.md) |
| 理解实现 | [架构](docs/zh-CN/ARCHITECTURE.md) |
| 对接客户端 | [接口](docs/zh-CN/reference/API.md)、[接收机](docs/zh-CN/reference/RECEIVERS.md)、[录制](docs/zh-CN/reference/RECORDINGS.md) |
| 发现信号与识别选定频段 | [扫描与识别](docs/zh-CN/reference/SIGNAL_SCAN.md) |
| 安装外部 provider | [工具](tools/README.md) |

`backend/` 负责服务及硬件边界，`dsp/` 负责共享信号处理内核，`frontend/` 负责浏览器交互，
`contracts/` 保存共享限制，`tests/` 和 `tools/` 保存回归样本及离线工具。

## 许可与致谢

有权授权的自有代码采用 [AGPL-3.0-only](LICENSE)，并附[第 7 条链接例外](LICENSE-EXCEPTION)，
**仅允许 HAROGIC SDK、FDK AAC 的指定组合**（`LicenseRef-Harogic-WebSA`）。自有代码及修改版本
保留适用的对应源码义务，包括修改网络版本的源码提供。界面提供许可文本及本地构建对应的源码入口。

第三方代码、库、模型和独立解码程序保留原许可，详见[适用范围及历史 MIT 授权](LICENSING.md)、
[第三方声明](THIRD_PARTY_NOTICES.md)及[核对依据与未决问题](docs/zh-CN/LICENSING.md)。
此前发布的 MIT 版本仍保留原 [MIT 授权](LICENSES/MIT-legacy.txt)。HAROGIC SDK 继续单独获取；
项目链接例外不授予厂商 SDK 的再分发权。

作者：[好奇牛马 (Bilibili)](https://space.bilibili.com/28447213)。
