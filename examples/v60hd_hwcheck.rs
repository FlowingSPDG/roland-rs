//! Full V-60HD LAN hardware check (all documented commands + tally/QPL).
//!
//! Usage: cargo run --example v60hd_hwcheck --features tokio -- 192.168.3.39

use std::time::Duration;

use roland_rs::devices::v60hd::{self, Channel, Composition, PanelStatus, Response};
use roland_rs::AsyncV60HdClient;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("V60HD_HOST").ok())
        .unwrap_or_else(|| "192.168.3.39".to_string());

    println!("== connect {host}:{} ==", v60hd::TELNET_PORT);
    let mut c = AsyncV60HdClient::connect(&host).await?;

    let (product, version) = c.ver().await?;
    println!("VER {product} {version}");

    if std::env::args().any(|a| a == "--restore") {
        let now = c.qpl_all().await?;
        println!("QPL before restore {now:?}");
        restore_toggles(&mut c, clean_panel(now)).await;
        println!("TLY {:?}", c.tly().await?);
        println!("QPL {:?}", c.qpl_all().await?);
        return Ok(());
    }

    let _acs = try_send(&mut c, "ACS", v60hd::acs()).await;
    let qal = try_send(&mut c, "QAL", v60hd::qal(v60hd::AudioLevelQuery::All)).await;

    let baseline_tly = c.tly().await?;
    println!("TLY baseline {baseline_tly:?}");
    let baseline = c.qpl_all().await?;
    println!("QPL baseline {baseline:?}");

    let _ = try_send(&mut c, "PST SDI2", v60hd::pst(Channel::Sdi2)).await;
    dump_unsolicited(&mut c, "after PST").await;
    println!("TLY after PST {:?}", c.tly().await?);
    println!("QPL after PST {:?}", c.qpl_all().await?);

    let _ = try_send(&mut c, "PGM SDI2", v60hd::pgm(Channel::Sdi2)).await;
    dump_unsolicited(&mut c, "after PGM").await;
    println!("TLY after PGM {:?}", c.tly().await?);
    println!("QPL after PGM {:?}", c.qpl_all().await?);

    let _ = try_send(&mut c, "AUX SDI3", v60hd::aux(Channel::Sdi3)).await;
    dump_unsolicited(&mut c, "after AUX").await;
    println!("QPL after AUX {:?}", c.qpl_all().await?);

    let _ = try_send(&mut c, "PGM SDI2 for CUT", v60hd::pgm(Channel::Sdi2)).await;
    let _ = try_send(&mut c, "PST SDI1 for CUT", v60hd::pst(Channel::Sdi1)).await;
    println!("QPL before CUT {:?}", c.qpl_all().await?);
    println!("TLY before CUT {:?}", c.tly().await?);
    let _ = try_send(&mut c, "CUT", v60hd::cut()).await;
    dump_unsolicited(&mut c, "after CUT").await;
    let tly_cut = c.tly().await?;
    let qpl_cut = c.qpl_all().await?;
    println!("TLY after CUT {tly_cut:?}");
    println!("QPL after CUT {qpl_cut:?}");
    let cut_ok = qpl_cut.pgm == Channel::Sdi1
        && qpl_cut.pst == Channel::Sdi2
        && tly_cut[0] == v60hd::TallyColor::Red
        && tly_cut[1] == v60hd::TallyColor::Green;
    println!(
        "{} CUT swap PGM=SDI1 PST=SDI2 TLY Red/Green",
        if cut_ok { "PASS" } else { "FAIL" }
    );

    let _ = try_send(
        &mut c,
        "TRS mix",
        v60hd::set_transition(v60hd::Transition::Mix),
    )
    .await;
    let _ = try_send(
        &mut c,
        "TIM 0.5s",
        v60hd::set_transition_time(v60hd::TransitionTime::new(5).unwrap()),
    )
    .await;
    let _ = try_send(&mut c, "PST SDI2 for AUTO", v60hd::pst(Channel::Sdi2)).await;
    let _ = try_send(&mut c, "ATO", v60hd::auto()).await;
    tokio::time::sleep(Duration::from_millis(700)).await;
    dump_unsolicited(&mut c, "after AUTO").await;
    println!("QPL after AUTO {:?}", c.qpl_all().await?);

    for (name, cmd) in [
        ("P1S", v60hd::pinp1_sw()),
        ("P1S restore", v60hd::pinp1_sw()),
        ("P2S", v60hd::pinp2_sw()),
        ("P2S restore", v60hd::pinp2_sw()),
        ("SPS", v60hd::split_sw()),
        ("SPS restore", v60hd::split_sw()),
        ("DSK", v60hd::dsk_sw()),
        ("DSK restore", v60hd::dsk_sw()),
        ("DVW", v60hd::dsk_pvw()),
        ("DVW restore", v60hd::dsk_pvw()),
        ("ATM", v60hd::auto_mixing()),
        ("ATM restore", v60hd::auto_mixing()),
        ("FDE", v60hd::output_fade()),
        ("FDE restore", v60hd::output_fade()),
    ] {
        let _ = try_send(&mut c, name, cmd).await;
        dump_unsolicited(&mut c, name).await;
        println!("QPL after {name} {:?}", c.qpl_all().await?);
    }
    println!("QPL after toggles {:?}", c.qpl_all().await?);
    restore_toggles(&mut c, clean_panel(baseline)).await;

    let _ = try_send(
        &mut c,
        "PP1 0,0",
        v60hd::set_pinp1_position(v60hd::PinPPosition::new(0, 0).unwrap()),
    )
    .await;
    let _ = try_send(
        &mut c,
        "PP2 0,0",
        v60hd::set_pinp2_position(v60hd::PinPPosition::new(0, 0).unwrap()),
    )
    .await;
    let _ = try_send(
        &mut c,
        "SPT 0,0",
        v60hd::set_split_position(v60hd::SplitPosition::new(0, 0).unwrap()),
    )
    .await;
    let _ = try_send(&mut c, "DSS SDI1", v60hd::set_dsk_source(Channel::Sdi1)).await;
    let _ = try_send(&mut c, "KYL 255", v60hd::set_dsk_key_level(255)).await;
    let _ = try_send(&mut c, "KYG 0", v60hd::set_dsk_key_gain(0)).await;
    let _ = try_send(
        &mut c,
        "IPS HDMI",
        v60hd::set_channel6_input(v60hd::Channel6Input::Hdmi),
    )
    .await;
    let _ = try_send(
        &mut c,
        "OS1 PGM",
        v60hd::set_sdi1_bus(v60hd::OutputBus::Program),
    )
    .await;
    let _ = try_send(
        &mut c,
        "OS2 PVW",
        v60hd::set_sdi2_bus(v60hd::OutputBus::Preview),
    )
    .await;
    let _ = try_send(
        &mut c,
        "OH1 PGM",
        v60hd::set_hdmi1_bus(v60hd::OutputBus::Program),
    )
    .await;
    let _ = try_send(
        &mut c,
        "OH2 AUX",
        v60hd::set_hdmi2_bus(v60hd::OutputBus::Aux),
    )
    .await;

    if let Ok(Response::AudioLevels { values }) = &qal {
        if let Some(master) = values.get(11).copied() {
            if let Ok(level) = v60hd::AudioLevel::from_tenths(master as i16) {
                let _ = try_send(&mut c, "OAL restore", v60hd::set_master_level(level)).await;
            }
        }
        if let Some(aux_lv) = values.get(12).copied() {
            if let Ok(level) = v60hd::AudioLevel::from_tenths(aux_lv as i16) {
                let _ = try_send(&mut c, "OAX restore", v60hd::set_aux_level(level)).await;
            }
        }
        if let Some(in1) = values.first().copied() {
            if let Ok(level) = v60hd::AudioLevel::from_tenths(in1 as i16) {
                let _ = try_send(
                    &mut c,
                    "IAL IN1 restore",
                    v60hd::set_input_audio_level(v60hd::AudioInput::AudioIn1, level),
                )
                .await;
            }
        }
    }
    let _ = try_send(
        &mut c,
        "ADT 0",
        v60hd::set_input_audio_delay(
            v60hd::AnalogAudioInput::AudioIn1,
            v60hd::AudioDelay::new(0).unwrap(),
        ),
    )
    .await;
    let _ = try_send(
        &mut c,
        "IAM IN1",
        v60hd::mute_input(v60hd::AudioInput::AudioIn1),
    )
    .await;
    let _ = try_send(
        &mut c,
        "IAM IN1 restore",
        v60hd::mute_input(v60hd::AudioInput::AudioIn1),
    )
    .await;
    let _ = try_send(
        &mut c,
        "IAS IN1",
        v60hd::solo_input(v60hd::AudioInput::AudioIn1),
    )
    .await;
    let _ = try_send(
        &mut c,
        "IAS IN1 restore",
        v60hd::solo_input(v60hd::AudioInput::AudioIn1),
    )
    .await;
    let _ = try_send(
        &mut c,
        "TPT off",
        v60hd::set_test_pattern(v60hd::TestPattern::Off),
    )
    .await;
    let _ = try_send(
        &mut c,
        "TTN off",
        v60hd::set_test_tone(v60hd::TestTone::Off),
    )
    .await;
    let _ = try_send(&mut c, "HCP off", v60hd::set_hdcp(v60hd::Hdcp::Off)).await;
    let _ = try_send(
        &mut c,
        "MEM 1",
        v60hd::load_memory(v60hd::MemorySlot::new(1).unwrap()),
    )
    .await;
    match timeout(
        Duration::from_secs(2),
        c.send(&v60hd::Command::custom("PGM", vec![99]).expect("opcode")),
    )
    .await
    {
        Ok(Ok(resp)) => println!("UNEXPECTED PGM:99 {resp:?}"),
        Ok(Err(e)) => println!("PASS PGM:99 rejected ({e})"),
        Err(_) => println!("FAIL PGM:99 timeout"),
    }

    let _ = try_send(&mut c, "restore PGM", v60hd::pgm(baseline.pgm)).await;
    let _ = try_send(&mut c, "restore PST", v60hd::pst(baseline.pst)).await;
    let _ = try_send(&mut c, "restore AUX", v60hd::aux(baseline.aux)).await;
    restore_toggles(&mut c, clean_panel(baseline)).await;
    let tly_final = c.tly().await?;
    let qpl_final = c.qpl_all().await?;
    println!("TLY final {tly_final:?}");
    println!("QPL final {qpl_final:?}");
    println!("== done ==");
    Ok(())
}

fn clean_panel(baseline: PanelStatus) -> PanelStatus {
    PanelStatus {
        composition: Composition::Off,
        dsk: false,
        output_fade: false,
        ..baseline
    }
}

async fn restore_toggles(client: &mut AsyncV60HdClient, target: PanelStatus) {
    for _ in 0..6 {
        let now = match client.qpl_all().await {
            Ok(panel) => panel,
            Err(e) => {
                println!("FAIL restore QPL {e}");
                return;
            }
        };
        if now.composition != target.composition {
            let name = match now.composition {
                Composition::Off => "restore composition on",
                _ => "restore composition off",
            };
            let cmd = match if now.composition != Composition::Off {
                now.composition
            } else {
                target.composition
            } {
                Composition::PinP1 => v60hd::pinp1_sw(),
                Composition::PinP2 => v60hd::pinp2_sw(),
                Composition::Split => v60hd::split_sw(),
                Composition::Off => continue,
            };
            let _ = try_send(client, name, cmd).await;
            continue;
        }
        if now.dsk != target.dsk {
            let _ = try_send(client, "restore DSK", v60hd::dsk_sw()).await;
            continue;
        }
        if now.output_fade != target.output_fade {
            let _ = try_send(client, "restore FDE", v60hd::output_fade()).await;
            continue;
        }
        println!("QPL restored {now:?}");
        return;
    }
    println!("FAIL restore toggles still mismatch");
}

async fn try_send(
    client: &mut AsyncV60HdClient,
    name: &str,
    cmd: v60hd::Command,
) -> Result<Response, String> {
    match timeout(Duration::from_secs(2), client.send(&cmd)).await {
        Ok(Ok(resp)) => {
            println!("OK {name} {resp:?}");
            Ok(resp)
        }
        Ok(Err(e)) => {
            println!("FAIL {name} {e}");
            Err(e.to_string())
        }
        Err(_) => {
            println!("TIMEOUT {name}");
            Err("timeout".into())
        }
    }
}

async fn dump_unsolicited(client: &mut AsyncV60HdClient, label: &str) {
    match timeout(Duration::from_millis(250), client.recv()).await {
        Ok(Ok(resp)) => println!("UNSOLICITED {label} {resp:?}"),
        Ok(Err(e)) => println!("UNSOLICITED {label} err {e}"),
        Err(_) => println!("UNSOLICITED {label} (none in 250ms)"),
    }
}
