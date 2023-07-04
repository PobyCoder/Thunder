use std::{time::Duration};
use log::info;
use rand::seq::SliceRandom;
use tokio::time::{sleep};
use crate::{game::{components::config::{config_component::Config}, systems::{matchs::random_match_wait_join_system::random_match_wait_join_start, message::system_message_system::system_message_send}}, database::redis::matchs::match_hash::{
    get_random_match_wait_list, get_match_join_user_list, get_match_wait_join_user_list, add_match, add_my_match, delete_match_wait_join_user_list, delete_match_join_user_list, delete_match_join_list, 
}};
use tokio::sync::Mutex as AsyncMutex;


pub async fn random_match_scheduler(shared_mutex: AsyncMutex<()>,config: Config) {
    let match_check_time = config.match_check_time.unwrap_or(1);
    let match_require_user_count = config.match_require_user_count.unwrap_or(2);
    let match_make_count_control = config.match_make_count_control.unwrap_or(0.1);
    let match_join_limit_time = config.match_join_limit_time.unwrap_or(10);
    let schedule_duration = Duration::from_secs(match_check_time.into());

    loop {
            {
                let _lock = shared_mutex.lock().await;
                let random_match_wait_list = get_random_match_wait_list().unwrap();

                let match_make_count = random_match_wait_list.len() as f64 / match_require_user_count as f64 * match_make_count_control;
                info!("match_make_count: {}", match_make_count.ceil() as usize);
                if random_match_wait_list.len() >= 2 {
                    for _ in 0..match_make_count.ceil() as usize {
                        let mut rng = rand::thread_rng();
                        let random_pick_user_list = random_match_wait_list
                            .choose_multiple(&mut rng, match_require_user_count as usize)
                            .collect::<Vec<_>>();

                        let random_pick_user_list_owned: Vec<String> = random_pick_user_list.iter().cloned().map(|s| s.to_owned()).collect();
                        let match_id = uuid::Uuid::new_v4().to_string();
                        random_match_wait_join_start(random_pick_user_list_owned,&match_id);
                        tokio::spawn(async move {
                            wait_match_join(match_join_limit_time,&match_id).await;
                        });
                    }
                }
            }

        sleep(schedule_duration).await;
    }
}

pub async fn wait_match_join(match_join_limit_time: u32, match_id: &String) {
    let total_duration = Duration::from_secs(match_join_limit_time as u64);
    let interval_duration = Duration::from_secs(1);
    let mut elapsed_duration = Duration::from_secs(0);

    while elapsed_duration < total_duration {
        println!("이벤트 발생!");
        let match_wait_join_user_list = get_match_wait_join_user_list(match_id).unwrap();
        let match_join_user_list = get_match_join_user_list(match_id).unwrap();
        if match_wait_join_user_list == match_join_user_list {
            info!("good");
            break;
        }
        sleep(interval_duration).await;
        elapsed_duration += interval_duration;
    }

    let match_wait_join_user_list = get_match_wait_join_user_list(match_id).unwrap();
    let match_join_user_list = get_match_join_user_list(match_id).unwrap();
    if match_wait_join_user_list == match_join_user_list {
        // match success
        for match_join_user in match_join_user_list {
            add_match(&match_id, &match_join_user).unwrap();
            add_my_match(&match_id, &match_join_user).unwrap();
            delete_match_join_list(&match_id).unwrap();
            system_message_send(&match_join_user, format!("All the User in the match have entered and the match({}) will begin.", match_id));
        }
    } else {
        // match fail
    }

    delete_match_wait_join_user_list(match_id).unwrap();
    delete_match_join_user_list(match_id).unwrap();

    // 타이머 종료 후 수행할 작업
    println!("타이머 종료");
}