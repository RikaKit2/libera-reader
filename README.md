
<div align="center">
  <p>
    <h1>Libera Reader 📚</h1>
    <strong>High-performance desktop e-book reader.</strong>
  </p>
  <p>
    <img src="https://img.shields.io/badge/License-AGPL_v3-blue.svg" alt="License">
    <img src="https://img.shields.io/badge/GUI-GPUI-blueviolet.svg" alt="GPUI">
    <img src="https://img.shields.io/badge/CI-Passed-success.svg" alt="CI Status">
  </p>
</div>

**Libera Reader** is a high-performance desktop application for reading and listening to e-books. Built with **Rust** and **GPUI**, it aims to provide the fastest and most reliable experience for managing large digital libraries.

### ✨ Key Features

* **Smart Library:** Real-time tracking of file changes (renames, moves, deletes) using `notify`.
* **Fast Indexing:** Parallel directory traversal with `jwalk` and sub-second metadata storage using `native-db` (redb).
* **Advanced Caching:** Instant cover loading through intelligent thumbnail management.
* **TTS Integration:** Support for offline (Piper-TTS) and online (Google API) voiceovers.

### 🛠 Tech Stack

* **Core:** Rust, native-db (redb), jwalk, notify, gxhash.
* **GUI:** GPUI (GPU-accelerated engine by Zed Industries).
* **Document Engine:** MuPDF via `mutool`.
* **Quality:** CI with Clippy, Security Audit, and Code Coverage.

### 📄 License

Distributed under the **AGPLv3** License.1
