use crate::compiler::Compiler;
use anyhow::{bail, Result};
use std::sync::Arc;
use target_lexicon::Triple;
use wasmtime_cranelift::isa_builder::IsaBuilder;
use wasmtime_environ::{CompilerBuilder, Setting, Tunables, RestoreInfo};
use winch_codegen::{isa, TargetIsa};

/// Compiler builder.
struct Builder {
    inner: IsaBuilder<Result<Box<dyn TargetIsa>>>,
    cranelift: Box<dyn CompilerBuilder>,
    tunables: Option<Tunables>,
    restore_info: Option<RestoreInfo>,
}

pub fn builder(triple: Option<Triple>) -> Result<Box<dyn CompilerBuilder>> {
    let inner = IsaBuilder::new(triple.clone(), |triple| {
        isa::lookup(triple).map_err(|e| e.into())
    })?;
    let cranelift = wasmtime_cranelift::builder(triple)?;
    Ok(Box::new(Builder {
        inner,
        cranelift,
        tunables: None,
        restore_info: None,
    }))
}

impl CompilerBuilder for Builder {
    fn triple(&self) -> &target_lexicon::Triple {
        self.inner.triple()
    }

    fn target(&mut self, target: target_lexicon::Triple) -> Result<()> {
        self.inner.target(target.clone())?;
        self.cranelift.target(target)?;
        Ok(())
    }

    fn set(&mut self, name: &str, value: &str) -> Result<()> {
        self.inner.set(name, value)?;
        self.cranelift.set(name, value)?;
        Ok(())
    }

    fn enable(&mut self, name: &str) -> Result<()> {
        self.inner.enable(name)?;
        self.cranelift.enable(name)?;
        Ok(())
    }

    fn settings(&self) -> Vec<Setting> {
        self.inner.settings()
    }

    fn set_tunables(&mut self, tunables: Tunables) -> Result<()> {
        if !tunables.winch_callable {
            bail!("Winch requires the winch calling convention");
        }

        if !tunables.table_lazy_init {
            bail!("Winch requires the table-lazy-init option to be enabled");
        }

        if !tunables.signals_based_traps {
            bail!("Winch requires the signals-based-traps option to be enabled");
        }

        if tunables.generate_native_debuginfo {
            bail!("Winch does not currently support generating native debug information");
        }

        self.tunables = Some(tunables.clone());
        self.cranelift.set_tunables(tunables)?;
        Ok(())
    }

    fn set_restore_info(&mut self, info: RestoreInfo) -> Result<()> {
        self.restore_info = Some(info.clone());
        Ok(())
    }

    fn build(&self) -> Result<Box<dyn wasmtime_environ::Compiler>> {
        let isa = self.inner.build()?;
        let cranelift = self.cranelift.build()?;
        let tunables = self
            .tunables
            .as_ref()
            .expect("set_tunables not called")
            .clone();
        let restore_info = self
            .restore_info
            .clone();
        Ok(Box::new(Compiler::new(isa, cranelift, tunables, restore_info)))
    }

    fn enable_incremental_compilation(
        &mut self,
        _cache_store: Arc<dyn wasmtime_environ::CacheStore>,
    ) -> Result<()> {
        bail!("incremental compilation is not supported on this platform");
    }
}

impl std::fmt::Debug for Builder {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Builder: {{ triple: {:?} }}", self.triple())
    }
}
