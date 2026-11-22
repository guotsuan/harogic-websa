//! Shared OnAir receiver geometry and frequency rules.
#[derive(Debug, Clone)]
pub struct OnAirCapability {
    pub mode: String,
    pub available: bool,
    pub unavailable_reason: Option<String>,
    pub input_rate_hz: u32,
    pub minimum_rate_hz: u32,
    pub bandwidth_hz: u32,
}

// APRS's Web slice is 192 kHz, but the network source has a 250 kHz floor.
// Tuples are (mode, exact adapter rate, minimum receiver rate, occupied bandwidth).
// OnAir v3.3.0 server/web/web_decoders.cpp (kSpecs/AdsbDec) fixes ADS-B at 2 MS/s.
// Its core/src/adsb/adsb_rx.cpp documents the narrower 2 MS/s frontend's
// sensitivity tradeoff. Our 0.9 MHz pass edge leaves transition room below
// the exact adapter's 1 MHz Nyquist; the wider receiver cadence is intentional.
pub(super) const MODES: [(&str, u32, u32, u32); 6] = [
    ("ais", 250_000, 250_000, 125_000),
    ("aprs", 250_000, 250_000, 20_000),
    ("ft4", 250_000, 250_000, 6_000),
    ("wspr", 250_000, 250_000, 6_000),
    ("meteor", 288_000, 288_000, 150_000),
    ("adsb", 2_000_000, 2_500_000, 1_800_000),
];

pub(super) const AIS_CENTER_HZ: u64 = 162_000_000;
const AIS_BENCH_ENV: &str = "WEBSA_ONAIR_AIS_BENCH_HZ";

pub(super) fn parse_ais_bench_frequency(value: Option<&str>) -> Result<Option<u64>, String> {
    let Some(value) = value else { return Ok(None); };
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{AIS_BENCH_ENV} must be an integer center frequency in Hz between 400000000 and 450000000"));
    }
    let center = value.parse::<u64>().ok().filter(|hz| (400_000_000..=450_000_000).contains(hz))
        .ok_or_else(|| format!("{AIS_BENCH_ENV} must be an integer center frequency in Hz between 400000000 and 450000000"))?;
    Ok(Some(center))
}

pub(super) fn source_virtual_frequency(mode: &str, center_hz: u64) -> Result<u64, String> {
    let bench = if mode == "ais" {
        let value = match std::env::var(AIS_BENCH_ENV) {
            Ok(value) => Some(value),
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(_)) => return Err(format!("{AIS_BENCH_ENV} must contain an integer frequency in Hz")),
        };
        parse_ais_bench_frequency(value.as_deref())?
    } else { None };
    validated_virtual_frequency(mode, center_hz, bench)
}

pub(super) fn virtual_frequency(center_hz: u64) -> u64 {
    ((center_hz + 500) / 1000) * 1000
}

pub fn validate_frequency(mode: &str, center_hz: u64) -> Result<(), String> {
    source_virtual_frequency(mode, center_hz).map(|_| ())
}

pub fn frequency_supported(mode: &str, center_hz: u64) -> bool {
    validate_frequency(mode, center_hz).is_ok()
}

/// Canonical center used when a decoder is selected while the receiver is tuned
/// outside the decoder's band.
pub fn default_frequency(mode: &str) -> Option<u64> {
    if mode == "ais" {
        // A configured bench center is the intended receive frequency for AIS.
        if let Ok(value) = std::env::var(AIS_BENCH_ENV) {
            if let Ok(Some(hz)) = parse_ais_bench_frequency(Some(&value)) { return Some(hz); }
        }
        return Some(AIS_CENTER_HZ);
    }
    Some(match mode {
        "adsb" => 1_090_000_000,
        "meteor" => 137_100_000,
        "aprs" => 144_800_000,
        "ft4" => 14_080_000,
        "wspr" => 14_095_600,
        _ => return None,
    })
}

/// True for the OnAir modes with a fixed RF band contract. Other external
/// providers (GNSS, RTTY, ISM) accept any tuned frequency.
pub fn band_limited(mode: &str) -> bool {
    default_frequency(mode).is_some()
}

pub(super) fn validated_virtual_frequency(mode: &str, center_hz: u64, bench: Option<u64>) -> Result<u64, String> {
    // Only the explicitly configured, exact bench center can relabel AIS IQ.
    if mode == "ais" && bench == Some(center_hz) && (400_000_000..=450_000_000).contains(&center_hz) {
        return Ok(AIS_CENTER_HZ);
    }
    let (low, high) = match mode {
        "ais" => (161_900_000, 162_100_000),
        "aprs" => (29_000_000, 470_000_000),
        "ft4" | "wspr" => (100_000, 450_000_000),
        "meteor" => (136_000_000, 138_500_000),
        "adsb" => (1_087_000_000, 1_093_000_000),
        _ => return Err(format!("unsupported OnAir decoder: {mode}")),
    };
    if center_hz < low || center_hz > high {
        return Err(format!("OnAir {mode} requires a frequency between {low} and {high} Hz"));
    }
    let virtual_hz = virtual_frequency(center_hz);
    if virtual_hz < low || virtual_hz > high {
        return Err(format!("OnAir {mode} rounded virtual frequency is outside its supported band"));
    }
    // Normal reception must keep the fixed maritime channel pair unchanged.
    if mode == "ais" && center_hz != AIS_CENTER_HZ {
        return Err("OnAir AIS requires 162000000 Hz, the center of both AIS channels".into());
    }
    Ok(virtual_hz)
}
