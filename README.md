# What is Vaultify ?
Vaultify is a cutting-edge vault manager tool designed to secure your sensitive data with robust AES-256 encryption. Built with Rust, it offers unparalleled performance and security.

Vaultify ensures a seamless user experience, making your information accessible only to you through an intuitive web interface. With Vaultify, you can easily manage your digital vaults and securely store your passwords with its integrated password manager functionality.

In addition to its standalone capabilities, Vaultify can be deployed as a server on a Network-Attached Storage (NAS) device or other server environments. This flexibility allows you to centralize your data management, providing secure access to your digital vaults across multiple devices within your network. By deploying Vaultify on a NAS, you can leverage the storage and processing power of your existing infrastructure, ensuring that your sensitive data remains secure and accessible only to authorized users via the web interface.

Keep your secrets secure in the digital age with Vaultify.

# Compile the frontend 

```shell
npm install

npm run build
```

# Lib to compile the backend of the project on Debian based Server
```shell
sudo apt-get update
          sudo apt-get install -y \
            build-essential \
            pkg-config \
            libgtk-3-dev \
            libgdk-pixbuf2.0-dev \
            libcairo2-dev \
            libatk1.0-dev \
            libpango1.0-dev \
            libglib2.0-dev \
            meson \
            ninja-build \
            javascriptcoregtk-4.1-dev \
            libsoup-3.0-dev \
            webkit2gtk-4.1-dev \
            libx11-dev \
            libxdo-dev
```

# Run the server

```shell
cargo build --release
cd target/
sudo release/s4-vaultify
```

# About US
We are four students from EPITA Lyon, and we created Vaultify as part of our S4 project. Our goal was to develop a secure and efficient vault manager using Rust. You can connect with us on LinkedIn:

* [Matteo Evola](https://www.linkedin.com/in/matteo-evola/)
* [Swann Charlery Fontes](https://www.linkedin.com/in/swann-charlery-fontes-682a71232/)
* [Lothaire Chacornac](https://www.linkedin.com/in/lothaire-chacornac/)
* [Paul Boucheret](https://www.linkedin.com/in/paul-boucheret/)
