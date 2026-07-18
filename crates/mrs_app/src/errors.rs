// SPDX-FileCopyrightText: 2025 Mikhail Suvorau (Suvorrow) <suvoraum@gmail.com>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug)]
pub struct SyggrelMrsError {
    pub message: String,
}

impl std::fmt::Display for SyggrelMrsError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for SyggrelMrsError {}
