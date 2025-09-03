use dialoguer::{Input, Password};
use dotenv::dotenv;
use std::{collections::HashMap, env};
use thirtyfour::{
    By, DesiredCapabilities, WebDriver,
    extensions::query,
    prelude::{ElementQueryable, ElementWaitable},
};
use tokio::time::{Duration, sleep};

const UNIT_SIZE: u32 = 25;
const FIRST_INDEX: u32 = 1813;
const SKIP_CATEGORY: u32 = 4;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // for develop
    dotenv().ok();

    // let user_id: String = Input::new().with_prompt("ユーザーID").interact().unwrap();
    // let password: String = Password::new()
    //     .with_prompt("パスワード")
    //     .interact()
    //     .unwrap();
    let user_id = env::var("USER_ID").unwrap();
    let password = env::var("PASSWORD").unwrap();
    let start: u32 = env::var("START_INDEX").unwrap().parse().unwrap();
    let end: u32 = env::var("END_INDEX").unwrap().parse().unwrap();

    let caps = DesiredCapabilities::chrome();
    let driver = WebDriver::new("http://localhost:34327", caps).await?;

    driver.goto("https://entrance.linguaporta.jp/").await?;
    let school_input = driver.find(By::Id("school")).await?;
    school_input.send_keys("鈴鹿工業高等専門学校").await?;
    let go_button = driver.find(By::Css("button[type=\"submit\"]")).await?;
    go_button.click().await?;

    // Auth Page
    let submit = driver
        .query(By::Css("button[type=\"submit\"]"))
        .first()
        .await?;

    submit.wait_until().displayed().await?;
    wait_sec().await;

    let user_input = driver.find(By::Css("input[type=\"text\"]")).await?;
    user_input.send_keys(user_id).await?;

    let password_input = driver.find(By::Css("input[type=\"password\"]")).await?;
    password_input.send_keys(password).await?;

    wait_sec().await;

    let submit = driver.find(By::Css("button[type=\"submit\"]")).await?;
    submit.click().await?;

    wait_sec().await;

    let script = "document.menu_form.main.value = 'study';document.menu_form.submit();";
    driver.execute(script, []).await.unwrap();

    wait_sec().await;

    driver.execute("select_reference(70)", []).await.unwrap();

    wait_sec().await;
    wait_sec().await;

    // Answer
    if start % UNIT_SIZE == 1 && end % UNIT_SIZE == 0 {
        let mut now_i = start;
        let mut start = start;
        let mut counter = (end - start + 1) / UNIT_SIZE;
        println!("start index count");
        while counter > 0 {
            println!("Category: {} - {}", start, start + UNIT_SIZE - 1);
            println!("{now_i}");
            select_question(&driver, now_i).await?;

            wait_sec().await;

            answer_question(&driver).await?;

            wait_sec().await;

            counter -= 1;
            start += UNIT_SIZE;
            now_i += SKIP_CATEGORY;
        }
    }

    wait_sec().await;
    //
    // Always explicitly close the browser.
    // driver.quit().await?;

    Ok(())
}

async fn wait_sec() {
    sleep(Duration::from_secs(1)).await;
}

async fn select_question(driver: &WebDriver, index: u32) -> Result<(), Box<dyn std::error::Error>> {
    let set_num = (index - 1) / 25 + 1;
    let index = FIRST_INDEX + ((set_num - 1) * SKIP_CATEGORY);

    let script = format!("select_unit('drill', '{}', '')", index);

    driver.execute(script, []).await?;

    Ok(())
}

async fn answer_question(driver: &WebDriver) -> Result<(), Box<dyn std::error::Error>> {
    let mut answers: HashMap<String, String> = HashMap::new();

    let clear_query = driver.query(By::ClassName("page-back-link"));
    while !clear_query.exists().await? {
        let selections = driver.find(By::Id("drill_form")).await?;
        let question = driver.find(By::Id("qu02")).await?.text().await?;

        let select = match answers.get(&question) {
            Some(a) => {
                selections
                    .find(By::Css(format!("input[value=\"{a}\"][type=\"radio\"]")))
                    .await?
            }
            None => {
                selections
                    .query(By::Css("input[type=\"radio\"]"))
                    .first()
                    .await?
            }
        };

        select.click().await?;
        let submit = driver.find(By::Id("ans_submit")).await?;
        submit.click().await?;

        wait_sec().await;

        let true_message = driver.query(By::Id("true_msg"));
        let false_message = driver.query(By::Id("false_msg"));

        if true_message.exists().await? {
            let next = driver
                .query(By::Css("input[type=\"submit\"][value=\"次の問題\"]"))
                .first()
                .await?;
            next.click().await?;

            wait_sec().await;
        } else if false_message.exists().await? {
            wait_sec().await;
            driver.execute("document.viewAnswer.submit();", []).await?;

            driver.query(By::Id("drill_form")).first().await?;

            let answer = driver.find(By::Id("drill_form")).await?;
            let answer = answer.text().await?;
            let answer = answer.split("：").last();

            if let Some(a) = answer {
                answers.insert(question, a.trim().to_string());
            }

            let next = driver
                .query(By::Css("input[type=\"submit\"][value=\"次の問題\"]"))
                .first()
                .await?;
            next.click().await?;
        }

        wait_sec().await;
    }

    driver.execute("document.back.submit();", []).await?;

    Ok(())
}
