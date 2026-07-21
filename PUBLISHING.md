# Publishing OdyCSUnmuter Beta

## 1. Replace one placeholder

Open `Cargo.toml` and replace:

```text
YOUR_GITHUB_USERNAME
```

with your GitHub username.

Keep the MIT license credited to **Ody**.

## 2. Create an empty GitHub repository

```text
Repository name: OdyCSUnmuter
Description: Restore globally muted player voice in downloaded CS2 demos.
Visibility: Public
```

Do not initialize another README, license, or `.gitignore`.

Suggested topics:

```text
cs2 counter-strike-2 source2 demo-parser faceit rust gotv voice-chat
```

## 3. Upload the source

Upload the CONTENTS of this folder, so `README.md`, `Cargo.toml`, `src`, and
`.github` appear at the repository root.

Suggested commit:

```text
Initial OdyCSUnmuter beta
```

Do not upload generated folders or demos.

## 4. Build the public no-Rust release

1. Open **Actions**.
2. Enable workflows if prompted.
3. Select **Build and publish OdyCSUnmuter Windows release**.
4. Click **Run workflow**.
5. Use:

```text
v0.1.0-beta.1
```

The workflow creates:

```text
OdyCSUnmuter-Beta-v0.1.0-beta.1-windows-x64.zip
OdyCSUnmuter-Beta-v0.1.0-beta.1-windows-x64.zip.sha256
```

## 5. If release creation lacks permission

Open:

```text
Settings -> Actions -> General -> Workflow permissions
```

Choose:

```text
Read and write permissions
```

Save and rerun the workflow.

## 6. Test the public download

Download the release ZIP into a clean folder and confirm:

1. It contains `OdyCSUnmuter.exe`.
2. It runs without Rust installed.
3. It generates `*_OdyCSunmuted.dem`.
4. The target appears with non-zero audio bytes.
5. The target is audible after the recommended CS2 commands.

Keep it marked as a pre-release while compatibility testing continues.
