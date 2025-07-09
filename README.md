# auto_morph

`auto_morph` is a Rust GUI SCP:RP morpher.

![image](https://github.com/user-attachments/assets/1da06b32-8fe8-4175-a5fa-4a8986365dda)




This guide is for **Windows users** who want to either download the prebuilt `.exe` or build it from source.

---

## 🚀 Option 1: Run Using Prebuilt Release Binary

1. Go to the [Releases](https://github.com/CottonDestroyer/auto_morph/releases) page.
2. Download the latest Windows binary:
   - Example: `auto_morph_standalone.exe`
3. (Optional) Move the `.exe` file to a convenient folder (like `C:\auto_morph\`).
4. Double-click the `.exe` to run, or use the Command Prompt:

   ```cmd
   cd C:\path\to\auto_morph\
   auto_morph.exe
   ```

---

## 🔧 Option 2: Build from Source (Using Cargo)

> 📌 Requires [Rust and Cargo](https://www.rust-lang.org/tools/install). You can install them using [rustup](https://rustup.rs/).

1. Open PowerShell or Command Prompt.
2. Clone the repository:

   ```powershell
   git clone https://github.com/CottonDestroyer/auto_morph.git
   cd auto_morph
   ```

3. Build the project in release mode:

   ```powershell
   cargo build --release
   ```

4. Run the compiled binary:

   ```powershell
   .\target\release\auto_morph.exe
   ```

---

## 🛠 Requirements

- **Rust** (via [rustup](https://rustup.rs/))
- **Git** (for cloning the repository)
- **Windows 10/11**, 64-bit

---

## 📄 License

This project is licensed under the [MIT License](LICENSE).

---

## 🤝 Contributing

Feel free to open issues or submit pull requests if you want to contribute or suggest features.
