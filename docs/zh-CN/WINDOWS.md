# Windows x64 运行与构建

Windows 10／11 x64 版使用与 Linux 版相同的浏览器前端。通过模拟模式可以先验证页面、频谱、瀑布图和内置 SDR 解调。硬件模式使用 HAROGIC 0.55.100 版 Windows x64 HTRA SDK。

## PX Z 手持式频谱仪

三款机型使用同一套 Windows x64 构建流程。厂商标称射频上限见下表。 [HAROGIC PX Z 产品页](https://www.harogic.cn/product/pxe-z-edition/)

| 机型 | 厂商标称射频上限 | WebSA 验证状态 |
| --- | ---: | --- |
| PXE-90 Z | 9.5 GHz | 9.4 GHz 的 SDR 调谐及回读已用模拟 SDK 验证，待实机验收 |
| PXE-200 Z | 20 GHz | 19.5 GHz 的 SDR 调谐及回读已用模拟 SDK 验证，待实机验收 |
| PXN-400Z | 40 GHz | 待确认本机 HTRA 连接方式及实机采集 |

WebSA 的适配器当前使用 HTRA USB 接口。如果厂商 SDK 只通过以太网暴露仪器，需适配以太网采集。确认连接时，请在手持机本机的 Windows 上先用厂商程序或 SDK 示例打开分析仪。退出厂商程序后再启动 WebSA。N400 R2V2 的实机结果来自已测试的那台设备，PXN-400Z 单独验收。

## 启动

在 Windows 应用目录双击 `Start-WebSA.bat` 启动实机模式。确保 HAROGIC 驱动已安装，SAStudio4 等占用仪器的程序已关闭，应用的 `htra_api` 下有厂商 SDK 运行库和这台仪器对应的 `CalFile`。从源码构建时，可先在仓库根目录运行下面的命令，把 SDK 运行库和校准目录复制到本地构建产物中。

```powershell
.\run-windows.ps1 -SdkDirectory 'C:\SDK\SA_Windows_API\x64\htra_api'
```

双击 BAT 后，在本机打开 [http://127.0.0.1:8080/](http://127.0.0.1:8080/)。服务监听 `0.0.0.0:8080`，局域网设备使用仪器电脑的实际 IP 访问。WebSA 没有登录认证，只在可信的专用网络内放行。按 Ctrl+C 停止服务。

模拟模式无需 SDK 或仪器。在仓库根目录运行 `.\run-windows.ps1 -Fake`。该开发脚本支持用 `-Port` 修改端口。录音和预设默认保存在 `%LOCALAPPDATA%\harogic-websa\recordings\<端口>`，也可用环境变量 `WEBSA_RECORDING_DIR` 指定目录。

## 从源码构建

工具链、SDK 路径和编译命令见[项目 README 的 Windows 小节](../../README.zh-CN.md#windows-x64-与-px-z-手持式频谱仪)。生成目录为 `windows-build\app`，可整体复制到另一台 Windows x64 电脑或手持机。复制前确认应用内的 `htra_api` 已准备好厂商运行库和目标仪器对应的校准文件。

省略构建命令中的 `-SdkDirectory` 可仅生成模拟模式所需文件。FDK AAC 使用 [v2.0.3 源码](https://github.com/mstorsjo/fdk-aac/tree/v2.0.3)静态链接。PowerShell 如禁止直接执行本地脚本，可在单次进程内启动。

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -File .\run-windows.ps1 -Fake
```

## 实机验证

先运行厂商示例确认驱动能识别仪器。关闭厂商程序后启动 WebSA，依次检查扫频、实时频谱和低速 IQ。切换模式和停止后确认设备可以重新打开，再从低速逐步提高 IQ 采样率，观察连续采集、丢包和溢出。Windows 走厂商 SDK 自带的 USB 采集路径，62.5 MS/s 的持续吞吐需要实机验证。

Windows 首版的 SoapySDRServer 接管、外部解码器和 Opus 编码在界面及 API 中报告不可用。内置解调与外部解码器是独立的。FLAC 和 Opus 录音的 PCM 试听需要 PATH 中有 FFmpeg。三种依赖 ONNX Runtime 的神经降噪模式需要兼容 1.23.2 的 x64 `onnxruntime.dll`，可将其放在 `websad.exe` 同目录，或用 `WEBSA_ORT_DYLIB` 指向 DLL。
