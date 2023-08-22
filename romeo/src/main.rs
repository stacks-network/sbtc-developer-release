use tokio::{join, sync::broadcast, time::sleep};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (sender, _) = broadcast::channel::<romeo::event::Event>(128);

    let sender_clone = sender.clone();
    tokio::spawn(async move {
        sleep(std::time::Duration::from_secs(10)).await;
        println!("Sending tick");
        sender_clone.send(romeo::event::Event::Tick).unwrap();
    });

    let h1 = romeo::actor::spawn::<romeo::deposit::DepositProcessor>(&sender);
    let h2 = romeo::actor::spawn::<romeo::contract_deployer::ContractDeployer>(&sender);

    let _ = join!(h1, h2);

    Ok(())
}
