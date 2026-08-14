# Android Build Setup

## Prerequisites

1. Install Android Studio or standalone SDK (API 35+)
2. Install `xbuild`:
   ```bash
   cargo install xbuild
   ```
3. Set environment:
   ```bash
   export ANDROID_HOME=$HOME/Android/Sdk  # or your SDK path
   export PATH=$PATH:$ANDROID_HOME/platform-tools
   ```

## Build & Run

```bash
# Build APK
x build --release --platform android --arch arm64

# Install on connected device
x run --platform android --arch arm64
```

## Current Status
- Desktop (Linux): Builds and runs
- Android: Config ready, needs SDK + xbuild installed
