use std::{collections::LinkedList, time::Duration};

use futures::{future, StreamExt};
use pid::PID;
use r2r::{
    geometry_msgs::msg::{Twist, Vector3},
    std_msgs::msg::Float32,
    ParameterValue, QosProfile,
};
use write_once::{RPtr, RWPtr, WPtr};

mod gui;
mod pid;
mod write_once;

pub struct App {
    pub k_p: RWPtr<f32, WPtr>,
    pub k_i: RWPtr<f32, WPtr>,
    pub k_d: RWPtr<f32, WPtr>,

    pub pid_linear: RWPtr<PID, RPtr>,
    pub pid_angular: RWPtr<PID, RPtr>,

    pub err_linear: RWPtr<LinkedList<f64>, RPtr>,
    pub err_angular: RWPtr<LinkedList<f64>, RPtr>,

    pub motor_gain: RWPtr<f32, WPtr>,
}

impl App {
    const FREQ: u64 = 20;
    const DT_MS: u64 = 1000 / Self::FREQ;
    const DT_SEC: f32 = Self::DT_MS as f32 / 1000.0;
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ctx = r2r::Context::create()?;
        let mut node = r2r::Node::create(ctx, "testnode", "")?;

        let pub_motor_1 =
            node.create_publisher::<Float32>("motor_1_power", QosProfile::default())?;
        let pub_motor_2 =
            node.create_publisher::<Float32>("motor_2_power", QosProfile::default())?;

        let sub_vel = node.subscribe::<Vector3>("velocity_vector", r2r::QosProfile::default())?;

        let (current_r, mut current_w) = write_once::new(Vector3::default());
        tokio::task::spawn(async move {
            sub_vel
                .for_each(|msg| {
                    *current_w = msg;
                    future::ready(())
                })
                .await;
        });

        let sub_cmd = node.subscribe::<Twist>("/cmd_vel", QosProfile::default())?;

        let (target, mut target_w) = write_once::new(Twist::default());
        // let target = WriteOnce::new(Twist::default());
        // let mut target_w = target.clone();

        tokio::task::spawn(async move {
            sub_cmd
                .for_each(|msg| {
                    *target_w = msg;
                    future::ready(())
                })
                .await;
        });

        let (k_p_r, k_p_w) = write_once::new(3.0);
        let (k_i_r, k_i_w) = write_once::new(0.001);
        let (k_d_r, k_d_w) = write_once::new(6.5);

        let pid_linear = PID::new(k_p_r.clone(), k_i_r.clone(), k_d_r.clone());
        let (pid_linear, mut pid_linear_w) = write_once::new(pid_linear);

        let pid_angular = PID::new(k_p_r.clone(), k_i_r.clone(), k_d_r.clone());
        let (pid_angular, mut pid_angular_w) = write_once::new(pid_angular);

        let (motor_gain_r, motor_gain) = write_once::new(1.0);

        let mut timer = node.create_wall_timer(Duration::from_millis(Self::DT_MS))?;

        std::thread::spawn(move || loop {
            node.spin_once(std::time::Duration::from_millis(100));
        });

        let (err_linear, mut err_linear_w) = write_once::new(LinkedList::new());

        let (err_angular, mut err_angular_w) = write_once::new(LinkedList::new());

        let mass = 1.0; // kg
        let motor_distance = 3.0; // meter

        tokio::task::spawn(async move {
            loop {
                let target_linear_speed = target.linear.x;
                let target_angular_speed = target.angular.z;

                let err_linear = target_linear_speed - current_r.x;
                err_linear_w.push_back(err_linear);

                let err_angular = target_angular_speed - current_r.z;
                err_angular_w.push_back(err_angular);

                while err_linear_w.len() > 1000 {
                    err_linear_w.pop_front();
                    err_angular_w.pop_front();
                }
                println!("lens => {} - {}", err_linear_w.len(), err_angular_w.len());

                let linear_correction = pid_linear_w.main(err_linear as f32, Self::DT_SEC);
                let angular_correction = pid_angular_w.main(err_angular as f32, Self::DT_SEC);

                let force = linear_correction * mass; // Vessel Mass
                let torque = angular_correction * mass * motor_distance;

                let left_motor_thrust = (force - torque) / 2.0;
                let right_motor_thrust = (force + torque) / 2.0;
                pub_motor_1
                    .publish(&Float32 {
                        data: left_motor_thrust.clamp(-*motor_gain_r, *motor_gain_r),
                    })
                    .unwrap();

                pub_motor_2
                    .publish(&Float32 {
                        data: right_motor_thrust.clamp(-*motor_gain_r, *motor_gain_r),
                    })
                    .unwrap();
                timer.tick().await.unwrap();
            }
        });

        Ok(Self {
            k_p: k_p_w,
            k_i: k_i_w,
            k_d: k_d_w,
            err_linear,
            err_angular,
            pid_angular,
            pid_linear,
            motor_gain,
        })
    }
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    let app = App::new().unwrap();
    eframe::run_native(
        "xosa_controller",
        options,
        Box::new(|_cc| Box::new(gui::Gui { app })),
    )
}

fn param_double(node: &r2r::Node, k: &str) -> Option<f64> {
    for x in node.params.lock().iter() {
        for (key, value) in x.iter() {
            match (key, value) {
                (key, ParameterValue::Double(res)) if key == k => {
                    return Some(*res);
                }
                _ => continue,
            };
        }
    }
    None
}
