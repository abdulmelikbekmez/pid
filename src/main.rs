use std::{collections::LinkedList, time::Duration};

use futures::{future, StreamExt};
use pid::PID;
use r2r::{
    geometry_msgs::msg::{Twist, Vector3},
    std_msgs::msg::Float32,
    QosProfile,
};
use write_once::WriteOnce;

mod gui;
mod pid;
mod write_once;

pub struct App {
    pub k_p: WriteOnce<f32>,
    pub k_i: WriteOnce<f32>,
    pub k_d: WriteOnce<f32>,

    pub err_linear: WriteOnce<LinkedList<f64>>,
    pub err_angular: WriteOnce<LinkedList<f64>>,
}

impl App {
    const FREQ: u32 = 20;
    const DT_MS: u32 = 1000 / Self::FREQ;
    const DT_SEC: f32 = Self::DT_MS as f32 / 1000.0;
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let ctx = r2r::Context::create()?;
        let mut node = r2r::Node::create(ctx, "testnode", "")?;

        let pub_motor_1 =
            node.create_publisher::<Float32>("motor_1_power", QosProfile::default())?;
        let pub_motor_2 =
            node.create_publisher::<Float32>("motor_2_power", QosProfile::default())?;

        let sub_vel = node.subscribe::<Vector3>("velocity_vector", r2r::QosProfile::default())?;

        let current = WriteOnce::new(Vector3::default());

        let mut tmp_w = current.clone();

        tokio::task::spawn(async move {
            sub_vel
                .for_each(|msg| {
                    *tmp_w = msg;
                    future::ready(())
                })
                .await;
        });

        let sub_cmd = node.subscribe::<Twist>("/cmd_vel", QosProfile::default())?;

        let target = WriteOnce::new(Twist::default());
        let mut target_w = target.clone();

        tokio::task::spawn(async move {
            sub_cmd
                .for_each(|msg| {
                    *target_w = msg;
                    future::ready(())
                })
                .await;
        });

        let k_p = WriteOnce::new(0.0);
        let k_p_c = k_p.clone();

        let k_i = WriteOnce::new(0.0);
        let k_i_c = k_i.clone();

        let k_d = WriteOnce::new(0.0);
        let k_d_c = k_d.clone();

        let mut pid_linear = PID::new(k_p_c, k_i_c, k_d_c);

        let k_p_c = k_p.clone();
        let k_i_c = k_i.clone();
        let k_d_c = k_d.clone();

        let mut pid_angular = PID::new(k_p_c, k_i_c, k_d_c);

        let mut timer = node.create_wall_timer(Duration::from_millis(1000 / Self::FREQ as u64))?;

        std::thread::spawn(move || loop {
            node.spin_once(std::time::Duration::from_millis(100));
        });

        let err_linear = WriteOnce::new(LinkedList::new());
        let mut err_linear_w = err_linear.clone();

        let err_angular = WriteOnce::new(LinkedList::new());
        let mut err_angular_w = err_angular.clone();

        let mass = 5.0; // kg
        let motor_distance = 3.0; // meter

        tokio::task::spawn(async move {
            loop {
                let target_linear_speed = target.linear.x;
                let target_angular_speed = target.angular.z;

                let err_linear = target_linear_speed - current.x;
                err_linear_w.push_back(err_linear);

                let err_angular = target_angular_speed - current.z;
                err_angular_w.push_back(err_angular);

                while err_linear_w.len() > 1000 {
                    err_linear_w.pop_front();
                    err_angular_w.pop_front();
                }
                println!("lens => {} - {}", err_linear_w.len(), err_angular_w.len());

                let linear_correction = pid_linear.main(err_linear as f32, 0.05);
                let angular_correction = pid_angular.main(err_angular as f32, 0.05);

                let force = linear_correction * mass; // Vessel Mass
                let torque = angular_correction * mass * motor_distance;

                let left_motor_thrust = (force - torque) / 2.0;
                let right_motor_thrust = (force + torque) / 2.0;
                pub_motor_1
                    .publish(&Float32 {
                        data: left_motor_thrust,
                    })
                    .unwrap();

                pub_motor_2
                    .publish(&Float32 {
                        data: right_motor_thrust,
                    })
                    .unwrap();
                timer.tick().await.unwrap();
            }
        });

        Ok(Self {
            k_p,
            k_i,
            k_d,
            err_linear,
            err_angular,
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
