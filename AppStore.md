# Packaging and Submitting to the Mac App Store

This guide details the prerequisites, configuration, packaging, and submission steps for releasing Daily Kanban (`dkb`) on the Mac App Store.

---

## 1. Prerequisites

- **Apple Developer Account**: Enrolled with Mac App Store distribution permissions.
- **Certificates**: Production signing certificates installed in your macOS Keychain:
  - `3rd Party Mac Developer Application: Your Name (TEAM_ID)` (or `Apple Distribution: Your Name (TEAM_ID)`)
  - `3rd Party Mac Developer Installer: Your Name (TEAM_ID)`
- **Provisioning Profile**: Mac App Store distribution provisioning profile for `com.doug.dkb` downloaded from the [Apple Developer portal](https://developer.apple.com/account/resources/profiles/list).

---

## 2. Entitlements & Info.plist Configuration

The application is configured for sandboxing:

- **`resources/Info.plist`**: Defines bundle identifier `com.doug.dkb`, application display name, high-resolution rendering, and deployment target macOS 13.0+.
- **`resources/dkb.entitlements`**:
  - `com.apple.security.app-sandbox`: Enabled (`true`)
  - `com.apple.security.files.user-selected.read-write`: Enabled (`true` for user-selected custom data storage directories)
  - `com.apple.security.files.bookmarks.app-scope`: Enabled (`true`)

---

## 3. Build and Sign the Installer Package (`.pkg`)

Use the packaging script to build the release binary, package the application bundle, embed the provisioning profile, sign with entitlements, and generate the signed installer package:

```bash
APP_KEY="3rd Party Mac Developer Application: Your Team Name (TEAM_ID)" \
INSTALLER_KEY="3rd Party Mac Developer Installer: Your Team Name (TEAM_ID)" \
PROVISIONING_PROFILE="path/to/DailyKanban.provisionprofile" \
./scripts/package_appstore.sh
```

### Artifacts Created:
- `target/release/bundle/Daily Kanban.app` (signed `.app` bundle with sandbox entitlements)
- `target/release/bundle/Daily Kanban.pkg` (signed installer package ready for App Store upload)

---

## 4. Validate and Upload to App Store Connect

You can upload the `.pkg` package using either the command line (`xcrun altool`) or GUI (`Transporter.app`).

### Option A: Command Line (`xcrun altool`)

1. **Validate the package**:
   ```bash
   xcrun altool --validate-app -f "target/release/bundle/Daily Kanban.pkg" \
     -t macos \
     -u "your-apple-id@example.com" \
     -p "@keychain:AC_PASSWORD"
   ```

2. **Upload the package**:
   ```bash
   xcrun altool --upload-app -f "target/release/bundle/Daily Kanban.pkg" \
     -t macos \
     -u "your-apple-id@example.com" \
     -p "@keychain:AC_PASSWORD"
   ```

> *Tip: Store your App Store Connect app-specific password in your Keychain using `xcrun altool --store-password-in-keychain-item "AC_PASSWORD" -u "your-apple-id@example.com" -p "xxxx-xxxx-xxxx-xxxx"`.*

### Option B: Transporter App
1. Download and open **Transporter** from the Mac App Store.
2. Drag and drop `target/release/bundle/Daily Kanban.pkg` into Transporter.
3. Click **Verify**, then click **Deliver**.

---

## 5. App Store Connect Release

1. Log in to [App Store Connect](https://appstoreconnect.apple.com).
2. Go to **Apps** > **Daily Kanban**.
3. Under the target version:
   - Select the uploaded build.
   - Provide description, keywords, support URL, and category (Productivity).
   - Upload screenshots for required macOS resolutions.
4. Submit the build for App Review.
