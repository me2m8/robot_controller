use gilrs::{Axis, Gilrs};
use serialport::{self, SerialPort};

const PORT: &str = "/dev/ttyACM0";
const BAUD_RATE: u32 = 9600;

const MOVE_SPEED: f64 = 50.0;

const MAX: u32 = 2400;
const MIN: u32 = 250;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Arm {
    pub base: f64,
    pub shoulder: f64,
    pub elbow: f64,
    pub claw: f64,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct RawArm {
    pub base: u16,
    pub shoulder: u16,
    pub elbow: u16,
    pub claw: u16,
}

impl Default for Arm {
    fn default() -> Arm {
        Arm {
            base: 90.0,
            shoulder: 0.0,
            elbow: 0.0,
            claw: 0.0,
        }
    }
}

impl Arm {
    fn to_raw(&self) -> RawArm {
        let delta = MAX - MIN;

        RawArm {
            base: ((self.base / 180.0) * delta as f64 + MIN as f64) as u16,
            shoulder: ((self.shoulder / 180.0) * delta as f64 + MIN as f64) as u16,
            elbow: ((self.elbow / 180.0) * delta as f64 + MIN as f64) as u16,
            claw: ((self.claw / 180.0) * delta as f64 + MIN as f64) as u16,
        }
    }
}

#[allow(dead_code)]
struct Connection {
    port: &'static str,
    baud_rate: u32,
    connection: Box<dyn SerialPort>,
}

fn main() {
    let mut connection = Connection::new(PORT, BAUD_RATE);
    let mut gilrs = Gilrs::new().unwrap();
    let mut arm = Arm::default();

    let mut claw_open = false;

    loop {
        if let Some(event) = gilrs.next_event() {
            let gamepad = gilrs.gamepad(event.id);

            let right_axis_y = gamepad.value(Axis::RightStickY) as f64;
            let left_axis_x = gamepad.value(Axis::LeftStickX) as f64;
            let left_axis_y = gamepad.value(Axis::LeftStickY) as f64;
            let a_button = gamepad.is_pressed(gilrs::Button::South);

            if a_button {
                claw_open = !claw_open;
                arm.claw = if claw_open { 180.0 } else { 0.0 };
            }

            arm.base = (arm.base + left_axis_x * MOVE_SPEED).clamp(0.0, 180.0);
            arm.shoulder = (arm.shoulder + right_axis_y * MOVE_SPEED).clamp(0.0, 180.0);
            arm.elbow = (arm.elbow + left_axis_y * MOVE_SPEED).clamp(0.0, 180.0);

            let data: [u8; 8] = unsafe { std::mem::transmute(arm.to_raw()) };

            connection.write(&data).unwrap();
        }
    }
}

impl Connection {
    fn new(port: &'static str, baud_rate: u32) -> Connection {
        let connection = serialport::new(port, baud_rate)
            .open()
            .expect("Failed to open port");
        Connection {
            port,
            baud_rate,
            connection,
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<(), serialport::Error> {
        let mut message: Vec<u8> = Vec::with_capacity(data.len() + 2);

        message.push(b'\r');
        for byte in data.into_iter() {
            message.push(*byte);
        }

        Ok(self.write(message.as_slice())?)
    }
}
