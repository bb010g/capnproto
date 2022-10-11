use anyhow::{anyhow, Context};
use cc::Build;
use std::env;

pub struct Cfg {
    name: String,
    propagated_definitions: Vec<(String, String, Option<String>)>,
}

impl Cfg {
    fn new(name: String) -> Self {
        Cfg {
            name,
            propagated_definitions: Default::default(),
        }
    }
}

impl Default for Cfg {
    fn default() -> Self {
        Cfg::new(env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME unset"))
    }
}

impl Cfg {
    pub fn define_propagated<'a, V: Into<Option<&'a str>>>(
        &mut self,
        var: &str,
        val: V,
    ) -> &mut Self {
        self.propagated_definitions.push((
            self.name.to_string(),
            var.to_string(),
            val.into().map(|s| s.to_string()),
        ));
        self
    }

    pub fn get_propagated_definition<'a>(&'a self, var: &str) -> Vec<(&'a str, Option<&'a str>)> {
        self.propagated_definitions
            .iter()
            .filter_map(|(dep, key, value)| {
                if key == var {
                    Some((&**dep, value.as_deref()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn import_propagated_definitions(&mut self) -> anyhow::Result<&mut Self> {
        for (key, value) in env::vars_os() {
            let key = match key.into_string() {
                Ok(key) => key,
                Err(_) => continue,
            };
            let dep = match key
                .strip_prefix("DEP_")
                .and_then(|key| key.strip_suffix("_KJ_BUILD_PROPAGATED_DEFINITIONS"))
            {
                Some("") => continue,
                Some(dep) => dep,
                None => continue,
            };
            let value = match value.into_string() {
                Ok(value) => value,
                Err(_) => continue,
            };
            let dep_propagated_definitions: Vec<(String, String, Option<String>)> =
                serde_json::from_str(&value)
                    .with_context(|| format!("propagated definitions from dependency {}", dep))?;
            self.propagated_definitions
                .extend(dep_propagated_definitions);
        }
        Ok(self)
    }

    pub fn export_propagated_definitions(
        &mut self,
        build: &mut Build,
    ) -> anyhow::Result<&mut Self> {
        println!(
            "cargo:kj-build-propagated-definitions={}",
            serde_json::to_string(&self.propagated_definitions)
                .with_context(|| anyhow!("propagated definitions serialization"))?
        );
        for (_dep, key, value) in self.propagated_definitions.drain(0..) {
            build.define(&key, value.as_deref());
        }
        Ok(self)
    }
}

pub trait BuildExt {
    fn propagate_definitions(&mut self, cfg: &mut Cfg) -> anyhow::Result<&mut Self>;
}

impl BuildExt for cc::Build {
    fn propagate_definitions(&mut self, cfg: &mut Cfg) -> anyhow::Result<&mut Self> {
        cfg.export_propagated_definitions(self)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
