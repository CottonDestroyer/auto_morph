# auto_morph

`auto_morph` is a Rust GUI SCP:RP morpher.

![image](https://github.com/user-attachments/assets/1da06b32-8fe8-4175-a5fa-4a8986365dda)





---

## 🚀 Option 1: Run Using Prebuilt Release Binary (Windows)

1. Go to the [Releases](https://github.com/CottonDestroyer/auto_morph/releases) page.
2. Download the latest Windows binary:
   - Example: `auto_morph-windows-x86_64.exe`
3. (Optional) Move the `.exe` file to a convenient folder (like `C:\auto_morph\`).
4. Double-click the `.exe` to run, or use the Command Prompt:

   ```cmd
   cd C:\path\to\auto_morph\
   auto_morph-windows-x86_64.exe
   ```

---

## 🏃 Run Using Prebuilt Release Binary (macOS)

1. **Download the release**  
   Head over to the [Releases](../../releases) page and download the zip containing the .app:  
   `auto_morph-macos.zip`

2. **Extract the archive**  
   You can extract it using Finder or via the terminal:
   ```sh
   unzip auto_morph-macos.zip
   ```

3. **Give execute permission (if needed)**  
   macOS might block the binary by default. Run:
   ```sh
   chmod +x auto_morph.app
   ```

4. **Run the application**
   You may need to install Rosetta if you're running this on silicon and it isn't already installed
   ```sh
   open -a auto_morph.app
   ```

> 🛡️ macOS Gatekeeper may block the app since it isn't signed. If prompted, go to **System Preferences > Security & Privacy**, and allow it manually under the "Security" tab.

---

## 🔧 Option 2: Build from Source (Windows)

> 📌 Requires [Rust and Cargo](https://www.rust-lang.org/tools/install). You can install them using [rustup](https://rustup.rs/).

1. Open PowerShell or Command Prompt.
2. Clone the repository:

   ```powershell
   git clone https://github.com/CottonDestroyer/auto_morph.git
   cd auto_morph
   ```

3. Build the project in release mode:

   ```powershell
   cargo build --release --bin auto_morph
   ```

4. Run the compiled binary:

   ```powershell
   .\target\release\auto_morph.exe
   ```

---

## 🛠️ How to Build from source (macOS)

1. **Install Rust**  
   If you haven't already, install Rust using [rustup](https://rustup.rs/):
   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Clone the repository**
   ```sh
   git clone https://github.com/CottonDestroyer/auto_morph.git
   cd auto_morph
   ```

3. **Build the project**
   ```sh
   cargo build --release --bin auto_morph
   ```

4. **Run the compiled binary**
   ```sh
   ./target/release/auto_morph
   ```

> ✅ This will produce a native macOS binary at `target/release/auto_morph`.

---

## 🛠 Requirements

- **Rust** (via [rustup](https://rustup.rs/))
- **Git** (for cloning the repository)
- **Windows 10/11** | **MacOS**
- 64-bit OS
---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

## 🤝 Contributing
@xokaiv (Mac OS implementation + Debug Log)

Feel free to open issues or submit pull requests if you want to contribute or suggest features.
