use crate::
{codegen::CodeGenError, 
isa::reg::Reg, masm::{MacroAssembler, StackSlot}};
use anyhow::{anyhow, Result};
use smallvec::SmallVec;
use wasmparser::{Ieee32, Ieee64};
use wasmtime_environ::WasmValType;
use std::collections::HashMap;
use log::error;

/// A typed register value used to track register values in the value
/// stack.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct TypedReg {
    /// The physical register.
    pub reg: Reg,
    /// The type associated to the physical register.
    pub ty: WasmValType,
}

impl TypedReg {
    /// Create a new [`TypedReg`].
    pub fn new(ty: WasmValType, reg: Reg) -> Self {
        Self { ty, reg }
    }

    /// Create an i64 [`TypedReg`].
    pub fn i64(reg: Reg) -> Self {
        Self {
            ty: WasmValType::I64,
            reg,
        }
    }

    /// Create an i32 [`TypedReg`].
    pub fn i32(reg: Reg) -> Self {
        Self {
            ty: WasmValType::I32,
            reg,
        }
    }

    /// Create an f64 [`TypedReg`].
    pub fn f64(reg: Reg) -> Self {
        Self {
            ty: WasmValType::F64,
            reg,
        }
    }

    /// Create an f32 [`TypedReg`].
    pub fn f32(reg: Reg) -> Self {
        Self {
            ty: WasmValType::F32,
            reg,
        }
    }

    /// Create a v128 [`TypedReg`].
    pub fn v128(reg: Reg) -> Self {
        Self {
            ty: WasmValType::V128,
            reg,
        }
    }
}

impl From<TypedReg> for Reg {
    fn from(tr: TypedReg) -> Self {
        tr.reg
    }
}

/// A local value.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Local {
    /// The index of the local.
    pub index: u32,
    /// The type of the local.
    pub ty: WasmValType,
}

/// A memory value.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub struct Memory {
    /// The type associated with the memory offset.
    pub ty: WasmValType,
    /// The stack slot corresponding to the memory value.
    pub slot: StackSlot,
}

/// Value definition to be used within the shadow stack.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) enum Val {
    /// I32 Constant.
    I32(i32),
    /// I64 Constant.
    I64(i64),
    /// F32 Constant.
    F32(Ieee32),
    /// F64 Constant.
    F64(Ieee64),
    /// V128 Constant.
    V128(i128),
    /// A register value.
    Reg(TypedReg),
    /// A local slot.
    Local(Local),
    /// Offset to a memory location.
    Memory(Memory),
}

impl From<TypedReg> for Val {
    fn from(tr: TypedReg) -> Self {
        Val::Reg(tr)
    }
}

impl From<Local> for Val {
    fn from(local: Local) -> Self {
        Val::Local(local)
    }
}
impl From<Memory> for Val {
    fn from(mem: Memory) -> Self {
        Val::Memory(mem)
    }
}

impl TryFrom<u32> for Val {
    type Error = anyhow::Error;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        i32::try_from(value).map(Val::i32).map_err(Into::into)
    }
}

impl Val {
    /// Create a new I32 constant value.
    pub fn i32(v: i32) -> Self {
        Self::I32(v)
    }

    /// Create a new I64 constant value.
    pub fn i64(v: i64) -> Self {
        Self::I64(v)
    }

    /// Create a new F32 constant value.
    pub fn f32(v: Ieee32) -> Self {
        Self::F32(v)
    }

    pub fn f64(v: Ieee64) -> Self {
        Self::F64(v)
    }

    /// Create a new V128 constant value.
    pub fn v128(v: i128) -> Self {
        Self::V128(v)
    }

    /// Create a new Reg value.
    pub fn reg(reg: Reg, ty: WasmValType) -> Self {
        Self::Reg(TypedReg { reg, ty })
    }

    /// Create a new Local value.
    pub fn local(index: u32, ty: WasmValType) -> Self {
        Self::Local(Local { index, ty })
    }

    /// Create a Memory value.
    pub fn mem(ty: WasmValType, slot: StackSlot) -> Self {
        Self::Memory(Memory { ty, slot })
    }

    /// Check whether the value is a register.
    pub fn is_reg(&self) -> bool {
        match *self {
            Self::Reg(_) => true,
            _ => false,
        }
    }

    /// Check whether the value is a memory offset.
    pub fn is_mem(&self) -> bool {
        match *self {
            Self::Memory(_) => true,
            _ => false,
        }
    }

    /// Check whether the value is a constant.
    pub fn is_const(&self) -> bool {
        match *self {
            Val::I32(_) | Val::I64(_) | Val::F32(_) | Val::F64(_) | Val::V128(_) => true,
            _ => false,
        }
    }

    /// Check whether the value is local
    pub fn is_local(&self) -> bool {
        match *self {
            Self::Local(_) => true,
            _ => false,
        }
    }

    /// Check whether the value is local with a particular index.
    pub fn is_local_at_index(&self, index: u32) -> bool {
        match *self {
            Self::Local(Local { index: i, .. }) if i == index => true,
            _ => false,
        }
    }

    /// Get the register representation of the value.
    ///
    /// # Panics
    /// This method will panic if the value is not a register.
    pub fn unwrap_reg(&self) -> TypedReg {
        match self {
            Self::Reg(tr) => *tr,
            v => panic!("expected value {v:?} to be a register"),
        }
    }

    /// Get the integer representation of the value.
    ///
    /// # Panics
    /// This method will panic if the value is not an i32.
    pub fn unwrap_i32(&self) -> i32 {
        match self {
            Self::I32(v) => *v,
            v => panic!("expected value {v:?} to be i32"),
        }
    }

    /// Get the integer representation of the value.
    ///
    /// # Panics
    /// This method will panic if the value is not an i64.
    pub fn unwrap_i64(&self) -> i64 {
        match self {
            Self::I64(v) => *v,
            v => panic!("expected value {v:?} to be i64"),
        }
    }

    /// Get the float representation of the value.
    ///
    /// # Panics
    /// This method will panic if the value is not an f32.
    pub fn unwrap_f32(&self) -> Ieee32 {
        match self {
            Self::F32(v) => *v,
            v => panic!("expected value {v:?} to be f32"),
        }
    }

    /// Get the float representation of the value.
    ///
    /// # Panics
    /// This method will panic if the value is not an f64.
    pub fn unwrap_f64(&self) -> Ieee64 {
        match self {
            Self::F64(v) => *v,
            v => panic!("expected value {v:?} to be f64"),
        }
    }

    /// Returns the underlying memory value if it is one, panics otherwise.
    pub fn unwrap_mem(&self) -> Memory {
        match self {
            Self::Memory(m) => *m,
            v => panic!("expected value {v:?} to be a Memory"),
        }
    }

    /// Check whether the value is an i32 constant.
    pub fn is_i32_const(&self) -> bool {
        match *self {
            Self::I32(_) => true,
            _ => false,
        }
    }

    /// Check whether the value is an i64 constant.
    pub fn is_i64_const(&self) -> bool {
        match *self {
            Self::I64(_) => true,
            _ => false,
        }
    }

    /// Check whether the value is an f32 constant.
    pub fn is_f32_const(&self) -> bool {
        match *self {
            Self::F32(_) => true,
            _ => false,
        }
    }

    /// Check whether the value is an f64 constant.
    pub fn is_f64_const(&self) -> bool {
        match *self {
            Self::F64(_) => true,
            _ => false,
        }
    }

    /// Get the type of the value.
    pub fn ty(&self) -> WasmValType {
        match self {
            Val::I32(_) => WasmValType::I32,
            Val::I64(_) => WasmValType::I64,
            Val::F32(_) => WasmValType::F32,
            Val::F64(_) => WasmValType::F64,
            Val::V128(_) => WasmValType::V128,
            Val::Reg(r) => r.ty,
            Val::Memory(m) => m.ty,
            Val::Local(l) => l.ty,
        }
    }
}

#[derive(Default, Debug)]
pub struct ModeStack {
    // innerの各値がreal/virtを判定するflagを管理. real=実スタックに現れる. virt=現れない(local.get, constなど)
    // real=true, virt=false
    inner: SmallVec<[bool; 64]>,
    // mode_sackのうちrealだけの数
    real_count: u32,
}

impl ModeStack {
    /// Allocate a new stack.
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            real_count: 0,
        }
    }
    
    // push
    pub fn push(&mut self, is_real: bool) {
       self.inner.push(is_real); 
       if is_real {
        self.real_count += 1;
       }
    }
    
    pub fn pop(&mut self) {
       let is_real = self.inner.pop().expect("Failed to pop mode stack"); 
       if is_real {
        self.real_count -= 1;
       }
    }
    
    pub fn truncate(&mut self, truncate: usize) {
       let pop_count = self.inner.len() - truncate;
       for _ in 0..pop_count {
        self.pop();
       }
    }
    
    pub fn get_real_count(&self) -> u32 {
        self.real_count
    }
}

/// The shadow stack used for compilation.
#[derive(Default, Debug)]
pub(crate) struct Stack {
    // NB: The 64 is chosen arbitrarily. We can adjust as we see fit.
    inner: SmallVec<[Val; 64]>,
    // innerの各値がreal/virtを判定するflagを管理. real=実スタックに現れる. virt=現れない(local.get, constなど)
    mode_stack: ModeStack,
    // TODO: 何のメタデータからわからない。(k, v) = (reg_id/mem_offset, position in stack)
    metadata: HashMap<u32, u8>,
}

impl Stack {
    /// Allocate a new stack.
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            mode_stack: Default::default(),
            metadata: HashMap::new(),
        }
    }

    /// Ensures that there are at least `n` elements in the value stack,
    /// and returns the index calculated by: stack length minus `n`.
    pub fn ensure_index_at(&self, n: usize) -> Result<usize> {
        if self.len() >= n {
            Ok(self.len() - n)
        } else {
            Err(anyhow!(CodeGenError::missing_values_in_stack()))
        }
    }

    /// Returns true if the stack contains a local with the provided index
    /// except if the only time the local appears is the top element.
    pub fn contains_latent_local(&self, index: u32) -> bool {
        self.inner
            .iter()
            // Iterate top-to-bottom so we can skip the top element and stop
            // when we see a memory element.
            .rev()
            // The local is not latent if it's the top element because the top
            // element will be popped next which materializes the local.
            .skip(1)
            // Stop when we see a memory element because that marks where we
            // spilled up to so there will not be any locals past this point.
            .take_while(|v| !v.is_mem())
            .any(|v| v.is_local_at_index(index))
    }

    /// Extend the stack with the given elements.
    pub fn extend(&mut self, values: impl IntoIterator<Item = Val>) {
        self.inner.extend(values);
    }

    /// Inserts many values at the given index.
    pub fn insert_many(&mut self, at: usize, values: &[Val]) {
        debug_assert!(at <= self.len());

        if at == self.len() {
            self.inner.extend_from_slice(values);
        } else {
            self.inner.insert_from_slice(at, values);
        }
    }

    /// Get the length of the stack.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Push a value to the stack.
    pub fn push(&mut self, val: Val) {
        error!("Not supported instruction");
        self.inner.push(val);
    }

    pub fn push_with_tag<M>(&mut self, masm: &mut M, val: Val) 
    where
        M: MacroAssembler,
    {
        if val.is_reg() {
            let reg = val.unwrap_reg();
            let addr = reg.reg.hw_enc() as u8;
            self.push_real_val(masm, val, addr).expect("failed to push reg");
        }
        else if val.is_mem() {
            error!("Not supported that val is mem");
        }
        else if val.is_const() || val.is_local() {
            self.push_virt_val(val);
        }
        else {
            error!("Failed to push_with_tag");
        }

        // self.inner.push(val);
    }

    fn push_virt_val(&mut self, val: Val) {
        self.inner.push(val);    
        self.mode_stack.push(false);
    }

    // addrにはreg.hw_enc()かメモリのオフセットが入る
    fn push_real_val<M>(&mut self, masm: &mut M, val: Val, addr: u8) -> Result<()> 
    where
        M: MacroAssembler,
    {
        self.inner.push(val);
        self.mode_stack.push(true);
        
        // real stackのpositionを埋め込む
        let stack_pos = self.get_real_stack_size();

        self.metadata.insert(stack_pos, addr);
        let _ = masm.store_metadata(stack_pos, addr as i32);
        Ok(())
    }

    pub fn truncate(&mut self, truncate: usize) {
        self.inner.truncate(truncate);
        self.mode_stack.truncate(truncate);
    }

    pub fn get_real_stack_size(&mut self) -> u32 {
        self.mode_stack.get_real_count()
    }

    pub fn get_metadata(&mut self, addr: u8) -> u32 {
        // self.metadata[&addr]
        if let Some((key, _)) = self.metadata.iter().find(|(_, &v)| v == addr) {
            return *key;
        }
        return u32::MAX;
    }

    pub fn metadata(&mut self) -> &HashMap<u32, u8> {
        &self.metadata
    }

    pub fn move_metadata<M>(&mut self, masm: &mut M, old_addr: u8, new_addr: u8) 
    where
        M: MacroAssembler,
    {
        // 実行時のメタデータ更新
        let value = self.get_metadata(old_addr);
        let _ = masm.store_metadata(value, new_addr as i32);

        // コンパイル時のメタデータ更新
        self.metadata.insert(value, new_addr);
    }

    /// Peek into the top in the stack.
    pub fn peek(&self) -> Option<&Val> {
        self.inner.last()
    }

    /// Returns an iterator referencing the last n items of the stack,
    /// in bottom-most to top-most order.
    pub fn peekn(&self, n: usize) -> impl Iterator<Item = &Val> + '_ {
        let len = self.len();
        assert!(n <= len);

        let partition = len - n;
        self.inner[partition..].into_iter()
    }

    /// Pops the top element of the stack, if any.
    pub fn pop(&mut self) -> Option<Val> {
        self.mode_stack.pop();

        self.inner.pop()
    }

    /// Pops the element at the top of the stack if it is an i32 const;
    /// returns `None` otherwise.
    pub fn pop_i32_const(&mut self) -> Option<i32> {
        match self.peek() {
            Some(v) => v.is_i32_const().then(|| self.pop().unwrap().unwrap_i32()),
            _ => None,
        }
    }

    /// Pops the element at the top of the stack if it is an i64 const;
    /// returns `None` otherwise.
    pub fn pop_i64_const(&mut self) -> Option<i64> {
        match self.peek() {
            Some(v) => v.is_i64_const().then(|| self.pop().unwrap().unwrap_i64()),
            _ => None,
        }
    }

    /// Pops the element at the top of the stack if it is an f32 const;
    /// returns `None` otherwise.
    pub fn pop_f32_const(&mut self) -> Option<Ieee32> {
        match self.peek() {
            Some(v) => v.is_f32_const().then(|| self.pop().unwrap().unwrap_f32()),
            _ => None,
        }
    }

    /// Pops the element at the top of the stack if it is an f64 const;
    /// returns `None` otherwise.
    pub fn pop_f64_const(&mut self) -> Option<Ieee64> {
        match self.peek() {
            Some(v) => v.is_f64_const().then(|| self.pop().unwrap().unwrap_f64()),
            _ => None,
        }
    }

    /// Pops the element at the top of the stack if it is a register;
    /// returns `None` otherwise.
    pub fn pop_reg(&mut self) -> Option<TypedReg> {
        match self.peek() {
            Some(v) => v.is_reg().then(|| self.pop().unwrap().unwrap_reg()),
            _ => None,
        }
    }

    /// Pops the given register if it is at the top of the stack;
    /// returns `None` otherwise.
    pub fn pop_named_reg(&mut self, reg: Reg) -> Option<TypedReg> {
        match self.peek() {
            Some(v) => {
                (v.is_reg() && v.unwrap_reg().reg == reg).then(|| self.pop().unwrap().unwrap_reg())
            }
            _ => None,
        }
    }

    /// Get a mutable reference to the inner stack representation.
    pub fn inner_mut(&mut self) -> &mut SmallVec<[Val; 64]> {
        &mut self.inner
    }

    /// Get a reference to the inner stack representation.
    pub fn inner(&self) -> &SmallVec<[Val; 64]> {
        &self.inner
    }

    /// Calculates the size of, in bytes, of the top n [Memory] entries
    /// in the value stack.
    pub fn sizeof(&self, top: usize) -> u32 {
        self.peekn(top).fold(0, |acc, v| {
            if v.is_mem() {
                acc + v.unwrap_mem().slot.size
            } else {
                acc
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Stack, Val};
    use crate::isa::reg::Reg;
    use wasmtime_environ::WasmValType;

    #[test]
    fn test_pop_i32_const() {
        let mut stack = Stack::new();
        stack.push(Val::i32(33i32));
        assert_eq!(33, stack.pop_i32_const().unwrap());

        stack.push(Val::local(10, WasmValType::I32));
        assert!(stack.pop_i32_const().is_none());
    }

    #[test]
    fn test_pop_reg() {
        let mut stack = Stack::new();
        let reg = Reg::int(2usize);
        stack.push(Val::reg(reg, WasmValType::I32));
        stack.push(Val::i32(4));

        assert_eq!(None, stack.pop_reg());
        let _ = stack.pop().unwrap();
        assert_eq!(reg, stack.pop_reg().unwrap().reg);
    }

    #[test]
    fn test_pop_named_reg() {
        let mut stack = Stack::new();
        let reg = Reg::int(2usize);
        stack.push(Val::reg(reg, WasmValType::I32));
        stack.push(Val::reg(Reg::int(4), WasmValType::I32));

        assert_eq!(None, stack.pop_named_reg(reg));
        let _ = stack.pop().unwrap();
        assert_eq!(reg, stack.pop_named_reg(reg).unwrap().reg);
    }
}
