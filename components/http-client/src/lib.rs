// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Used for `header!` macro which cannot be correctly resolved as it is exported as `hyper::header`
// which is also a module name.
#[macro_use]
extern crate hyper;
// Convenience importing of `debug!`/`info!` macros for entire crate.
#[macro_use]
extern crate log;

mod api_client;
mod error;
mod net;
pub mod proxy;
pub mod util;

pub use crate::api_client::ApiClient;
pub use crate::error::{Error, Result};

#[cfg(not(target_os = "macos"))]
mod ssl {
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use std::str::FromStr;

    use habitat_core::env;
    use habitat_core::fs::cache_ssl_path;
    use habitat_core::package::{PackageIdent, PackageInstall};
    use openssl::ssl::SslContextBuilder;

    use crate::error::Result;

    const CACERTS_PKG_IDENT: &str = "core/cacerts";
    const CACERT_PEM: &str = include_str!(concat!(env!("OUT_DIR"), "/cacert.pem"));

    pub fn set_ca(ctx: &mut SslContextBuilder, fs_root_path: Option<&Path>) -> Result<()> {
        let cacerts_ident = PackageIdent::from_str(CACERTS_PKG_IDENT)?;

        if env::var("SSL_CERT_FILE").is_ok() || env::var("SSL_CERT_DIR").is_ok() {
            ctx.set_default_verify_paths()?;
        } else if let Ok(pkg_install) = PackageInstall::load(&cacerts_ident, fs_root_path) {
            let pkg_certs = pkg_install.installed_path().join("ssl/cert.pem");
            debug!(
                "Setting CA file for SSL context to: {}",
                pkg_certs.display()
            );
            ctx.set_ca_file(pkg_certs)?;
        } else {
            let cached_certs = cache_ssl_path(fs_root_path).join("cert.pem");
            if !cached_certs.exists() {
                fs::create_dir_all(cache_ssl_path(fs_root_path))?;
                debug!("Creating cached cacert.pem at: {}", cached_certs.display());
                let mut file = File::create(&cached_certs)?;
                file.write_all(CACERT_PEM.as_bytes())?;
            }
            debug!(
                "Setting CA file for SSL context to: {}",
                cached_certs.display()
            );
            ctx.set_ca_file(cached_certs)?;
        }
        Ok(())
    }
}

#[cfg(target_os = "macos")]
mod ssl {
    use std::path::Path;

    use openssl::ssl::SslContextBuilder;

    use crate::error::Result;

    pub fn set_ca(ctx: &mut SslContextBuilder, _fs_root_path: Option<&Path>) -> Result<()> {
        ctx.set_default_verify_paths()?;
        Ok(())
    }
}
