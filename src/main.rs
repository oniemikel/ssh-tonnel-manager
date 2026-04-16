use serde::{Serialize, Deserialize};
use dialoguer::{theme::ColorfulTheme, Select, Input};
use std::process::Command;
use std::io::{self, Write};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct SshConfig {
    name: String,
    user: String,
    ip: String,
    port: String,
    key_path: String,
    local_port: String,
    target: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct AppConfig {
    hosts: Vec<SshConfig>,
}

const APP_NAME: &str = "ssh-tunnel-manager";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    loop {
        // 設定の読み込み（初回は空のリストが作成される）
        let mut cfg: AppConfig = confy::load(APP_NAME, None)?;

        let menu = vec!["接続する", "新しい接続を追加", "接続を削除", "終了"];
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("メニューを選択してください")
            .items(&menu)
            .default(0)
            .interact()?;

        match selection {
            0 => connect_tunnel(&cfg.hosts)?,
            1 => {
                let new_host = add_host_prompt();
                cfg.hosts.push(new_host);
                confy::store(APP_NAME, None, cfg)?;
                println!("保存しました。");
            },
            2 => {
                if cfg.hosts.is_empty() { println!("削除できる接続がありません。"); continue; }
                let del_idx = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("削除する接続を選択")
                    .items(&cfg.hosts.iter().map(|h| &h.name).collect::<Vec<_>>())
                    .interact()?;
                cfg.hosts.remove(del_idx);
                confy::store(APP_NAME, None, cfg)?;
            },
            _ => break,
        }
    }
    Ok(())
}

fn add_host_prompt() -> SshConfig {
    SshConfig {
        name: Input::new().with_prompt("名前 (例: My-OCI)").interact_text().unwrap(),
        user: Input::new().with_prompt("ユーザー名").default("opc".into()).interact_text().unwrap(),
        ip: Input::new().with_prompt("パブリックIP").interact_text().unwrap(),
        port: Input::new().with_prompt("SSHポート").default("22".into()).interact_text().unwrap(),
        key_path: Input::new().with_prompt("秘密鍵パス").interact_text().unwrap(),
        local_port: Input::new().with_prompt("ローカルポート").interact_text().unwrap(),
        target: Input::new().with_prompt("ターゲット (default localhost:PORT)").default("".into()).interact_text().unwrap(),
    }
}

fn connect_tunnel(hosts: &[SshConfig]) -> Result<(), Box<dyn std::error::Error>> {
    if hosts.is_empty() {
        println!("接続先が登録されていません。");
        return Ok(());
    }

    let idx = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("接続先を選択")
        .items(&hosts.iter().map(|h| &h.name).collect::<Vec<_>>())
        .interact()?;

    let h = &hosts[idx];
    let target = if h.target.is_empty() { format!("localhost:{}", h.local_port) } else { h.target.clone() };

    println!("--- SSHトンネル起動中: {} ---", h.name);
    
    let mut child = Command::new("ssh")
        .args(&["-N", "-L", &format!("{}:{}", h.local_port, target), "-p", &h.port, "-i", &h.key_path, &format!("{}@{}", h.user, h.ip)])
        .spawn()?;

    println!("Enterキーで切断します...");
    let mut _buf = String::new();
    io::stdin().read_line(&mut _buf)?;
    
    child.kill()?;
    println!("切断完了。");
    Ok(())
}