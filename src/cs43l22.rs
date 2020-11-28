use stm32f4xx_hal::hal::{
    blocking::i2c::{Write, WriteRead},
    digital::v2::OutputPin
};

const CS43L22_ADDRESS: u8 = 0x4A;
pub struct CS43L22<RESET, I2C> where {  
    i2c: I2C,
    reset: RESET
}

impl <RESET, I2C, I2CERR> CS43L22<RESET, I2C> 
where 
    RESET: OutputPin, 
    I2C: Write<Error = I2CERR> + WriteRead<Error = I2CERR>,
    I2CERR: core::fmt::Debug 
{
    pub fn new(reset: RESET, i2c: I2C) -> Self {
        Self { reset, i2c }
    }

    pub fn initialize(&mut self) {
        let _ = self.reset.set_high();

        let mut i2c_read_buffer = [0];

        let cs43l22_id = match self.i2c.write_read(CS43L22_ADDRESS, &[0x01], &mut i2c_read_buffer[0..1]) {
            Ok(()) => Some(i2c_read_buffer[0]),
            _ => None
        };
    
        assert!(cs43l22_id == Some(0xE3));
    
        self.i2c.write(CS43L22_ADDRESS, &[0x02, 0x01]).unwrap();
    
        // Commence the boot sequence
        self.i2c.write(CS43L22_ADDRESS, &[0x00, 0x99]).unwrap();
        self.i2c.write(CS43L22_ADDRESS, &[0x47, 0x80]).unwrap();
        self.i2c.write(CS43L22_ADDRESS, &[0x32, 0x80]).unwrap();
        self.i2c.write(CS43L22_ADDRESS, &[0x32, 0x00]).unwrap();
        self.i2c.write(CS43L22_ADDRESS, &[0x00, 0x00]).unwrap();
    
        // Set to headphones
        self.i2c.write(CS43L22_ADDRESS, &[0x04, 0xAF]).unwrap();
    
        // Set the power control high
        self.i2c.write(CS43L22_ADDRESS, &[0x02, 0x9E]).unwrap();
    
        // Set volume
        self.i2c.write(CS43L22_ADDRESS, &[0x20, 0x90]).unwrap();
        self.i2c.write(CS43L22_ADDRESS, &[0x21, 0x90]).unwrap();
        self.i2c.write(CS43L22_ADDRESS, &[0x1a, 0]).unwrap();
        self.i2c.write(CS43L22_ADDRESS, &[0x1b, 0]).unwrap();
    }
}
