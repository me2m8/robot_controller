use serialport::{self, SerialPort};
use gilrs::{Axis, Gilrs};

const PORT: &str = "/dev/ttyACM0";
const BAUD_RATE: u32 = 9600;

const MOVE_SPEED: f64 = 100.0;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct Arm {
    pub base: f64,
    pub shoulder: f64,
    pub elbow: f64,
    pub claw: f64,
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

            let data: [u8; 32] = unsafe { std::mem::transmute::<Arm, [u8; 32]>(arm.clone()) };

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

        // if (Instant::now() - self.last_write) > Duration::from_millis(10) {
        //     self.last_write = Instant::now();
        // } else {
        //     println!("Ratelimiting ({}s left)", (Instant::now() - self.last_write).as_secs_f32());
        //     Err(ComError::Ratelimit)
        // }
        Ok(self.write(message.as_slice())?)
    }
}
