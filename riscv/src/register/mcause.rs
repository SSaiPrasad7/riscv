//! mcause register

use riscv_pac::CoreInterruptNumber;
pub use riscv_pac::{ExceptionNumber, InterruptNumber}; // re-export useful riscv-pac traits

/// mcause register
#[derive(Clone, Copy, Debug)]
pub struct Mcause {
    bits: usize,
}

impl From<usize> for Mcause {
    #[inline]
    fn from(bits: usize) -> Self {
        Self { bits }
    }
}

/// Trap Cause
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
}

/// Interrupt
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Interrupt {
    SupervisorSoft = 1,
    MachineSoft = 3,
    SupervisorTimer = 5,
    MachineTimer = 7,
    SupervisorExternal = 9,
    MachineExternal = 11,
    Unknown,
}

/// Exception
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum Exception {
    InstructionMisaligned = 0,
    InstructionFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadMisaligned = 4,
    LoadFault = 5,
    StoreMisaligned = 6,
    StoreFault = 7,
    UserEnvCall = 8,
    SupervisorEnvCall = 9,
    MachineEnvCall = 11,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
    Unknown,
}

impl From<usize> for Interrupt {
    #[inline]
    fn from(nr: usize) -> Self {
        if nr > 11 || nr % 2 == 0 {
            Self::Unknown
        } else {
            // SAFETY: valid interrupt number
            unsafe { core::mem::transmute::<usize, Self>(nr) }
        }
    }
}

impl TryFrom<Interrupt> for usize {
    type Error = Interrupt;

    #[inline]
    fn try_from(value: Interrupt) -> Result<Self, Self::Error> {
        match value {
            Interrupt::Unknown => Err(Self::Error::Unknown),
            _ => Ok(value as Self),
        }
    }
}

/// SAFETY: `Interrupt` represents the standard RISC-V interrupts
unsafe impl InterruptNumber for Interrupt {
    const MAX_INTERRUPT_NUMBER: u16 = Self::MachineExternal as u16;

    #[inline]
    fn number(self) -> u16 {
        self as u16
    }

    #[inline]
    fn from_number(value: u16) -> Result<Self, u16> {
        match (value as usize).into() {
            Self::Unknown => Err(value),
            value => Ok(value),
        }
    }
}

/// SAFETY: `Interrupt` represents the standard RISC-V core interrupts
unsafe impl CoreInterruptNumber for Interrupt {}

impl From<usize> for Exception {
    #[inline]
    fn from(nr: usize) -> Self {
        if nr == 10 || nr == 14 || nr > 15 {
            Self::Unknown
        } else {
            // SAFETY: valid exception number
            unsafe { core::mem::transmute::<usize, Self>(nr) }
        }
    }
}

impl TryFrom<Exception> for usize {
    type Error = Exception;

    #[inline]
    fn try_from(value: Exception) -> Result<Self, Self::Error> {
        match value {
            Exception::Unknown => Err(Self::Error::Unknown),
            _ => Ok(value as Self),
        }
    }
}

/// SAFETY: `Exception` represents the standard RISC-V exceptions
unsafe impl ExceptionNumber for Exception {
    const MAX_EXCEPTION_NUMBER: u16 = Self::StorePageFault as u16;

    #[inline]
    fn number(self) -> u16 {
        self as u16
    }

    #[inline]
    fn from_number(value: u16) -> Result<Self, u16> {
        match (value as usize).into() {
            Self::Unknown => Err(value),
            value => Ok(value),
        }
    }
}

impl Mcause {
    /// Returns the contents of the register as raw bits
    #[inline]
    pub fn bits(&self) -> usize {
        self.bits
    }

    /// Returns the code field
    #[inline]
    pub fn code(&self) -> usize {
        self.bits & !(1 << (usize::BITS as usize - 1))
    }

    /// Trap Cause
    #[inline]
    pub fn cause(&self) -> Trap {
        if self.is_interrupt() {
            Trap::Interrupt(Interrupt::from(self.code()))
        } else {
            Trap::Exception(Exception::from(self.code()))
        }
    }

    /// Is trap cause an interrupt.
    #[inline]
    pub fn is_interrupt(&self) -> bool {
        self.bits & (1 << (usize::BITS as usize - 1)) != 0
    }

    /// Is trap cause an exception.
    #[inline]
    pub fn is_exception(&self) -> bool {
        !self.is_interrupt()
    }
}

read_csr_as!(Mcause, 0x342);

#[cfg(test)]
mod test {
    use super::*;
    use Exception::*;
    use Interrupt::*;

    #[test]
    fn test_interrupt() {
        assert_eq!(Interrupt::from_number(1), Ok(SupervisorSoft));
        assert_eq!(Interrupt::from_number(2), Err(2));
        assert_eq!(Interrupt::from_number(3), Ok(MachineSoft));
        assert_eq!(Interrupt::from_number(4), Err(4));
        assert_eq!(Interrupt::from_number(5), Ok(SupervisorTimer));
        assert_eq!(Interrupt::from_number(6), Err(6));
        assert_eq!(Interrupt::from_number(7), Ok(MachineTimer));
        assert_eq!(Interrupt::from_number(8), Err(8));
        assert_eq!(Interrupt::from_number(9), Ok(SupervisorExternal));
        assert_eq!(Interrupt::from_number(10), Err(10));
        assert_eq!(Interrupt::from_number(11), Ok(MachineExternal));
        assert_eq!(Interrupt::from_number(12), Err(12));

        assert_eq!(SupervisorSoft.number(), 1);
        assert_eq!(MachineSoft.number(), 3);
        assert_eq!(SupervisorTimer.number(), 5);
        assert_eq!(MachineTimer.number(), 7);
        assert_eq!(SupervisorExternal.number(), 9);
        assert_eq!(MachineExternal.number(), 11);

        assert_eq!(MachineExternal.number(), Interrupt::MAX_INTERRUPT_NUMBER)
    }

    #[test]
    fn test_exception() {
        assert_eq!(Exception::from_number(0), Ok(InstructionMisaligned));
        assert_eq!(Exception::from_number(1), Ok(InstructionFault));
        assert_eq!(Exception::from_number(2), Ok(IllegalInstruction));
        assert_eq!(Exception::from_number(3), Ok(Breakpoint));
        assert_eq!(Exception::from_number(4), Ok(LoadMisaligned));
        assert_eq!(Exception::from_number(5), Ok(LoadFault));
        assert_eq!(Exception::from_number(6), Ok(StoreMisaligned));
        assert_eq!(Exception::from_number(7), Ok(StoreFault));
        assert_eq!(Exception::from_number(8), Ok(UserEnvCall));
        assert_eq!(Exception::from_number(9), Ok(SupervisorEnvCall));
        assert_eq!(Exception::from_number(10), Err(10));
        assert_eq!(Exception::from_number(11), Ok(MachineEnvCall));
        assert_eq!(Exception::from_number(12), Ok(InstructionPageFault));
        assert_eq!(Exception::from_number(13), Ok(LoadPageFault));
        assert_eq!(Exception::from_number(14), Err(14));
        assert_eq!(Exception::from_number(15), Ok(StorePageFault));
        assert_eq!(Exception::from_number(16), Err(16));

        assert_eq!(InstructionMisaligned.number(), 0);
        assert_eq!(InstructionFault.number(), 1);
        assert_eq!(IllegalInstruction.number(), 2);
        assert_eq!(Breakpoint.number(), 3);
        assert_eq!(LoadMisaligned.number(), 4);
        assert_eq!(LoadFault.number(), 5);
        assert_eq!(StoreMisaligned.number(), 6);
        assert_eq!(StoreFault.number(), 7);
        assert_eq!(UserEnvCall.number(), 8);
        assert_eq!(SupervisorEnvCall.number(), 9);
        assert_eq!(MachineEnvCall.number(), 11);
        assert_eq!(InstructionPageFault.number(), 12);
        assert_eq!(LoadPageFault.number(), 13);
        assert_eq!(StorePageFault.number(), 15);

        assert_eq!(StorePageFault.number(), Exception::MAX_EXCEPTION_NUMBER)
    }
}
