use std::sync::{Mutex, MutexGuard, OnceLock};

use numbat_exchange_rates::{parse_exchange_rates, ExchangeRates};

static EXCHANGE_RATES: OnceLock<Mutex<Option<ExchangeRates>>> = OnceLock::new();
static WRITE_CACHE: Mutex<Option<Box<dyn Fn(&ExchangeRates) + Send + 'static>>> = Mutex::new(None);
static READ_CACHE: Mutex<Option<Box<dyn Fn() -> Option<ExchangeRates> + Send + 'static>>> =
    Mutex::new(None);

pub struct ExchangeRatesCache {}

impl ExchangeRatesCache {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_rate(&self, currency: &str) -> Option<f64> {
        let rates = Self::fetch();
        rates.as_ref().and_then(|r| r.get(currency)).cloned()
    }

    pub fn set_from_xml(xml_content: &str) {
        EXCHANGE_RATES
            .set(Mutex::new(parse_exchange_rates(xml_content)))
            .unwrap();
    }

    #[cfg(feature = "fetch-exchangerates")]
    pub fn set_hooks(
        write: Box<dyn Fn(&ExchangeRates) + Send + 'static>,
        read: Box<dyn Fn() -> Option<ExchangeRates> + Send + 'static>,
    ) {
        WRITE_CACHE.lock().unwrap().replace(write);
        READ_CACHE.lock().unwrap().replace(read);
    }

    #[cfg(feature = "fetch-exchangerates")]
    pub fn fetch() -> MutexGuard<'static, Option<ExchangeRates>> {
        if let Some(rates) = EXCHANGE_RATES.get() {
            return rates.lock().unwrap()
        }

        if let Some(read_cache) = READ_CACHE.lock().unwrap() {
            if let Some(rates) = read_cache() {
                EXCHANGE_RATES.set(rates);
                return EXCHANGE_RATES.get().unwrap().lock().unwrap()
            }
        }
        EXCHANGE_RATES
            .get_or_init(|| Mutex::new(numbat_exchange_rates::fetch_exchange_rates()))
            .lock()
            .unwrap()
            .map(|rates| {
                if let Some(write_cache) = WRITE_CACHE.lock().unwrap() {
                    write_cache(&rates)
                }
            })
    }

    #[cfg(not(feature = "fetch-exchangerates"))]
    pub fn fetch() -> MutexGuard<'static, Option<ExchangeRates>> {
        EXCHANGE_RATES.get().unwrap().lock().unwrap()
    }
}
