//! # serial_komunikator
//! Knižnica na komunikáciu s ESP32 cez sériový port.
//! Podporuje príkazy pre riadenie premietania (spustenie, vypnutie,
//! prepínanie piesní/slôh, zatmavenie obrazovky).

use serialport::SerialPort;
use std::io::{self, BufRead, BufReader, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Maximálna veľkosť jedného kusu správy (znaky)
const MAX_CHUNK: usize = 31;

/// Stav premietania – sleduje aktuálnu pieseň, slohu a stav zariadenia.
#[derive(Debug, Clone)]
pub struct StavPremietania {
    pub cislo_pesnicky: i32,
    pub cislo_slohy: i32,
    pub blackscreen: bool,
    pub premietanie_bezi: bool,
}

impl StavPremietania {
    pub fn new() -> Self {
        StavPremietania {
            cislo_pesnicky: 0,
            cislo_slohy: 0,
            blackscreen: false,
            premietanie_bezi: false,
        }
    }
}

impl Default for StavPremietania {
    fn default() -> Self {
        Self::new()
    }
}

/// Príkazy, ktoré môžeš odoslať na ESP32.
#[derive(Debug, Clone)]
pub enum Prikaz {
    /// %$1$0$% – spustenie premietania
    SpustiPremietanie,
    /// %$2$0$% – vypnutie premietania
    VypniPremietanie,
    /// %$3$0$% – pošli pieseň (číslo piesne)
    PosliPiesen(i32),
    /// %$4$0$% – zatmavenie obrazovky
    Zatmav,
    /// %$5$0$% – odtmavenie obrazovky
    Odtmav,
    /// %|(x)|(y)|% – prepnutie na pieseň (x) a slohu (y)
    PrepniNa { piesen: i32, sloha: i32 },
}

impl Prikaz {
    /// Serializuje príkaz na reťazec podľa protokolu.
    pub fn na_retazec(&self) -> String {
        match self {
            Prikaz::SpustiPremietanie => "%$1$0$%".to_string(),
            Prikaz::VypniPremietanie => "%$2$0$%".to_string(),
            Prikaz::PosliPiesen(x) => format!("%$3${}$%", x),
            Prikaz::Zatmav => "%$4$0$%".to_string(),
            Prikaz::PrepniNa { piesen, sloha } => format!("%|{}|{}|%", piesen, sloha),
            Prikaz::Odtmav => "%$5$0$%".to_string(),
        }
    }
}

/// Hlavný komunikátor – drží zápis na port a stav premietania.
#[derive(Debug)]
pub struct Komunikator {
    write_port: Box<dyn SerialPort>,
    pub stav: StavPremietania,
}

impl Komunikator {
    /// Otvorí sériový port a vráti nový `Komunikator`.
    ///
    /// # Príklad
    /// ```no_run
    /// let mut kom = serial_komunikator::Komunikator::pripoj("/dev/ttyUSB0", 115200).unwrap();
    /// ```
    pub fn pripoj(port_name: &str, baud_rate: u32) -> Result<Self, serialport::Error> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(500))
            .open()?;
        Ok(Komunikator {
            write_port: port,
            stav: StavPremietania::new(),
        })
    }

    /// Spustí vlákno, ktoré číta správy z ESP32 a volá `on_sprava` pre každý riadok.
    /// Vráti `JoinHandle` pre prípadné čakanie.
    ///
    /// # Príklad
    /// ```no_run
    /// let mut kom = serial_komunikator::Komunikator::pripoj("/dev/ttyUSB0", 115200).unwrap();
    /// kom.spusti_citanie(|sprava| println!("[ESP32] {}", sprava));
    /// ```
    pub fn spusti_citanie<F>(&self, on_sprava: F) -> thread::JoinHandle<()>
    where
        F: Fn(String) + Send + 'static,
    {
        let read_port = self
            .write_port
            .try_clone()
            .expect("Nepodarilo sa naklonovať port");
        thread::spawn(move || {
            let reader = BufReader::new(read_port);
            for line in reader.lines() {
                match line {
                    Ok(l) if !l.is_empty() => on_sprava(l),
                    _ => {}
                }
            }
        })
    }

    /// Odošle surový textový reťazec (rozdelí na kusy ak je dlhý).
    pub fn odosli_retazec(&mut self, msg: &str) -> io::Result<()> {
        let bytes = msg.as_bytes();
        let chunks: Vec<&str> = bytes
            .chunks(MAX_CHUNK)
            .map(|c| std::str::from_utf8(c).unwrap_or(""))
            .collect();

        let total = chunks.len();
        for (i, chunk) in chunks.iter().enumerate() {
            let data = format!("{}\n", chunk);
            match self.write_port.write_all(data.as_bytes()) {
                Ok(_) => {
                    #[cfg(debug_assertions)]
                    eprintln!("[Odoslané {}/{}] {}", i + 1, total, chunk);
                }
                Err(e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("[CHYBA] Zápis zlyhal: {}", e);
                    return Err(e);
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        Ok(())
    }

    /// Odošle štruktúrovaný príkaz na ESP32 a aktualizuje interný stav.
    ///
    /// # Príklad
    /// ```no_run
    /// use serial_komunikator::{Komunikator, Prikaz};
    /// let mut kom = Komunikator::pripoj("/dev/ttyUSB0", 115200).unwrap();
    /// kom.odosli_prikaz(Prikaz::SpustiPremietanie).unwrap();
    /// kom.odosli_prikaz(Prikaz::PrepniNa { piesen: 3, sloha: 2 }).unwrap();
    /// ```
    pub fn odosli_prikaz(&mut self, prikaz: Prikaz) -> io::Result<()> {
        // Aktualizuj stav podľa príkazu
        match &prikaz {
            Prikaz::SpustiPremietanie => {
                self.stav.premietanie_bezi = true;
                self.stav.blackscreen = false;
            }
            Prikaz::VypniPremietanie => {
                self.stav.premietanie_bezi = false;
            }
            Prikaz::PosliPiesen(x) => {
                self.stav.cislo_pesnicky = *x;
                self.stav.cislo_slohy = 0;
            }
            Prikaz::Zatmav => {
                self.stav.blackscreen = true;
            }
            Prikaz::PrepniNa { piesen, sloha } => {
                self.stav.cislo_pesnicky = *piesen;
                self.stav.cislo_slohy = *sloha;
                //self.stav.blackscreen = false;
            }
            Prikaz::Odtmav => {
                self.stav.blackscreen = false;
            }
        }

        let retazec = prikaz.na_retazec();
        self.odosli_retazec(&retazec)
    }

    /// Pohodlná metóda – prepne na nasledujúcu slohu tej istej piesne.
    pub fn dalsia_sloha(&mut self) -> io::Result<()> {
        let piesen = self.stav.cislo_pesnicky;
        let sloha = self.stav.cislo_slohy + 1;
        self.odosli_prikaz(Prikaz::PrepniNa { piesen, sloha })
    }

    /// Pohodlná metóda – prepne na predchádzajúcu slohu (min. 0).
    pub fn predchadzajuca_sloha(&mut self) -> io::Result<()> {
        let piesen = self.stav.cislo_pesnicky;
        let sloha = (self.stav.cislo_slohy - 1).max(0);
        self.odosli_prikaz(Prikaz::PrepniNa { piesen, sloha })
    }

    /// Pohodlná metóda – prepne na nasledujúcu pieseň (sloha sa resetuje na 0).
    pub fn dalsia_piesen(&mut self) -> io::Result<()> {
        let piesen = self.stav.cislo_pesnicky + 1;
        self.odosli_prikaz(Prikaz::PrepniNa { piesen, sloha: 0 })
    }

    /// Pohodlná metóda – prepne na predchádzajúcu pieseň (min. 0).
    pub fn predchadzajuca_piesen(&mut self) -> io::Result<()> {
        let piesen = (self.stav.cislo_pesnicky - 1).max(0);
        self.odosli_prikaz(Prikaz::PrepniNa { piesen, sloha: 0 })
    }
}

/// Verzia komunikátora pre prostredie s viacerými vláknami (Arc<Mutex<>>).
pub type KomunikatorMtx = Arc<Mutex<Komunikator>>;

/// Vytvorí `Arc<Mutex<Komunikator>>` – použiteľné z viacerých vlákien.
pub fn pripoj_mtx(port_name: &str, baud_rate: u32) -> Result<KomunikatorMtx, serialport::Error> {
    let kom = Komunikator::pripoj(port_name, baud_rate)?;
    Ok(Arc::new(Mutex::new(kom)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prikazy_serializacia() {
        assert_eq!(Prikaz::SpustiPremietanie.na_retazec(), "%$1$0$%");
        assert_eq!(Prikaz::VypniPremietanie.na_retazec(), "%$2$0$%");
        assert_eq!(Prikaz::PosliPiesen(5).na_retazec(), "%$3$5$%");
        assert_eq!(Prikaz::Zatmav.na_retazec(), "%$4$0$%");
        assert_eq!(
            Prikaz::PrepniNa {
                piesen: 3,
                sloha: 2
            }
            .na_retazec(),
            "%|3|2|%"
        );
    }

    #[test]
    fn test_stav_default() {
        let stav = StavPremietania::default();
        assert_eq!(stav.cislo_pesnicky, 0);
        assert_eq!(stav.cislo_slohy, 0);
        assert!(!stav.blackscreen);
        assert!(!stav.premietanie_bezi);
    }
}

/// Automaticky nájde prvý ESP32 na sériovom porte podľa VID čipov.
/// Podporuje: CP2102 (0x10C4), CH340/CH9102 (0x1A86), natívne USB Espressif (0x303A).
pub fn najdi_esp32() -> Option<String> {
    let porty = serialport::available_ports().ok()?;
    for port in &porty {
        if let serialport::SerialPortType::UsbPort(info) = &port.port_type {
            let vid = info.vid;
            if vid == 0x10C4 || vid == 0x1A86 || vid == 0x303A {
                return Some(port.port_name.clone());
            }
        }
    }
    None
}
