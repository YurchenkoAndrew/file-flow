# File Flow

**File Flow** is a fast, cross-platform desktop application designed for deep filesystem analysis, disk cleanup, and finding duplicate files. Built with modern, high-performance technologies, it helps you easily reclaim storage space and organize your data.

---

## 🚀 Tech Stack

*   **Core / Backend:** [Rust](https://www.rust-lang.org/) (High-performance native filesystem operations, multi-threaded hashing)
*   **Desktop Framework:** [Tauri 2.0](https://tauri.app/) (Secure, lightweight native wrapper)
*   **Frontend:** [Angular](https://angular.dev/) (v22+) + [Angular Material](https://material.angular.io/) (Modern UI components)

---

## ✨ Features

*   **Disk & Folder Scanner:** Analyze directory structures, track total file sizes, and view item counts.
*   **Category Breakdown:** Visual statistics of your storage usage grouped by file categories (Documents, Media, Archives, etc.).
*   **Heavy Files Analysis:** Quickly locate and inspect the largest files on your storage.
*   **Duplicate Finder:** Multi-threaded MD5-based verification to accurately detect and group duplicate files, helping you safely clean up wasted space.
*   **Native OS Integration:** Open files or reveal them directly highlighted in your system's file manager (Explorer / Finder / File Manager).
*   **Dark / Light ThemeService:** Seamless theme support based on system preferences.

---

## 🛠️ Getting Started (Development)

Make sure you have the following installed on your system:
*   [Node.js](https://nodejs.org/) & npm
*   [Rust toolchain](https://www.rust-lang.org/tools/install)
*   [Tauri CLI Prerequisites](https://v2.tauri.app/start/prerequisites/)

### 1. Clone the repository
```bash
git clone [https://github.com/your-username/file-flow.git](https://github.com/your-username/file-flow.git)
cd file-flow