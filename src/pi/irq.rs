use super::IO_BASE;
use super::mbox;
use volatile::prelude::*;
use volatile::{Volatile, ReadVolatile, WriteVolatile};

/*
#define GPIO ((volatile __attribute__((aligned(4))) struct GPIORegisters*)(uintptr_t)(RPi_IO_Base_Addr + 0x200000))
#define SYSTEMTIMER ((volatile __attribute__((aligned(4))) struct SystemTimerRegisters*)(uintptr_t)(RPi_IO_Base_Addr + 0x3000))
#define IRQ ((volatile __attribute__((aligned(4))) struct IrqControlRegisters*)(uintptr_t)(RPi_IO_Base_Addr + 0xB200))
#define ARMTIMER ((volatile __attribute__((aligned(4))) struct  ArmTimerRegisters*)(uintptr_t)(RPi_IO_Base_Addr + 0xB400))
#define MAILBOX ((volatile __attribute__((aligned(4))) struct MailBoxRegisters*)(uintptr_t)(RPi_IO_Base_Addr + 0xB880))
*/

const IRQ_BASE: usize = IO_BASE + 0xB200;

#[repr(C)]
struct Registers {
    IRQ_basic_pending: Volatile<u32>,
    IRQ_pending: [Volatile<u32>; 2],
    FIQ_control: Volatile<u32>,

    Enable_IRQs: [Volatile<u32>; 2],
    EnableBasicIRQs: Volatile<u32>,

    DisableIRQs: [Volatile<u32>; 2],
    DisableBasicIRQs: Volatile<u32>,
}

pub fn enable() {
    unsafe {
        let mut x = &mut *(0x40000040 as *mut Volatile<u32>);
        x.write(0);
    }


    let reg = unsafe { &mut *(IRQ_BASE as *mut Registers) };
    //reg.EnableBasicIRQs.write(0xFFFF_FFFF);
    //reg.Enable_IRQs[0].write(1 << 5);
    //reg.Enable_IRQs[0].write(0xFFFF_FFFF);
    //reg.Enable_IRQs[1].write(0xF000_0000);

    /*
    let reg = unsafe { &mut *(ARM_TIMER_BASE as *mut ArmTimerReg) };

    reg.Load.write(1000 - 1);
    reg.Reload.write(1000 - 1);
    reg.IRQ_ClearAck.write(0);
    reg.PreDivider.write(0xF9);
    //reg.Control.write(0x3E0020 | (1 << 7) | (1 << 5) | (1 << 1));
    reg.Control.write(0x3E00A2);


    //ARMTIMER->Load = divisor;										// Set the load value to divisor
    //ARMTIMER->Control.Counter32Bit = true;							// Counter in 32 bit mode
    //ARMTIMER->Control.Prescale = Clkdiv1;							// Clock divider = 1
    //ARMTIMER->Control.TimerIrqEnable = true;						// Enable timer irq
    //ARMTIMER->Control.TimerEnable = true;							// Now start the clock

    // enable interrupts
    // */
    unsafe { asm!("msr daifclr, #2" :::: "volatile") }
    println!("XXXX");
}

//pub fn timer_irq_setup(period_us: u32, handler: usize) {
 //   divisor

/*
TimerIrqHandler TimerIrqSetup (uint32_t period_in_us,				// Period between timer interrupts in usec
                            TimerIrqHandler ARMaddress)          // Function to call on interrupt
{
    uint32_t divisor;
    uint32_t Buffer[5] = { 0 };
    TimerIrqHandler OldHandler;
    ARMTIMER->Control.TimerEnable = false;							// Make sure clock is stopped, illegal to change anything while running
    mailbox_tag_message(&Buffer[0], 5, MAILBOX_TAG_GET_CLOCK_RATE,
        8, 8, 4, Buffer[4]);										// Get GPU clock (it varies between 200-450Mhz)
    Buffer[4] /= 250;												// The prescaler divider is set to 250 (based on GPU=250MHz to give 1Mhz clock)
    divisor = ((uint64_t)period_in_us*Buffer[4]) / 1000000;			// Divisor we would need at current clock speed
    OldHandler = setTimerIrqAddress(ARMaddress);					// Set new interrupt handler
    IRQ->EnableBasicIRQs.Enable_Timer_IRQ = true;					// Enable the timer interrupt IRQ
    ARMTIMER->Load = divisor;										// Set the load value to divisor
    ARMTIMER->Control.Counter32Bit = true;							// Counter in 32 bit mode
    ARMTIMER->Control.Prescale = Clkdiv1;							// Clock divider = 1
    ARMTIMER->Control.TimerIrqEnable = true;						// Enable timer irq
    ARMTIMER->Control.TimerEnable = true;							// Now start the clock
    return OldHandler;												// Return last function pointer	
}
*/

const ARM_TIMER_BASE: usize = 0x3E00B000 + 0x400;

pub struct ArmTimerReg {
    pub Load: Volatile<u32>,
    pub Value: ReadVolatile<u32>,
    pub Control: Volatile<u32>,
    pub IRQ_ClearAck: WriteVolatile<u32>,
    pub RAW_IRQ: ReadVolatile<u32>,
    pub MaskedIRQ: ReadVolatile<u32>,
    pub Reload: Volatile<u32>,
    pub PreDivider: Volatile<u32>,
    pub FreeRunningCounter: Volatile<u32>,
}

pub struct ArmTimer {
    pub reg: &'static mut ArmTimerReg,
}

impl ArmTimer {
    pub fn new() -> Self {
        let reg = unsafe { &mut *(ARM_TIMER_BASE as *mut ArmTimerReg) };
        reg.Control.write(0x3E0020 | 1 << 7 | 1 << 5);
        reg.Load.write(3000);
        reg.Reload.write(3000);
        reg.IRQ_ClearAck.write(0x1234);
        println!("free: {}", reg.FreeRunningCounter.read());
        Self {
            reg,
        }
    }

    pub fn load(&mut self, value: u32) {
        self.reg.Load.write(value);
    }
}


