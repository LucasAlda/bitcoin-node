use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::str::FromStr;

use crate::error::CustomError;

#[derive(Debug)]

/// Config es una estructura que contiene los valores de configuracion del nodo.
/// Estos valores se leen de un archivo de configuracion.
/// Los valores son:
/// - seed: semilla DNS para obtener direcciones IP.
/// - protocol_version: version del protocolo.
/// - port: puerto en el que escucha el nodo.
pub struct Config {
    pub seed: String,
    pub protocol_version: i32,
    pub port: u16,
    pub log_file: String,
    pub npeers: u8,
    pub client_only: bool,
    pub store_path: String,
}

impl Config {
    /// Lee un archivo de configuracion y devuelve un Config con los valores leidos.
    /// El archivo de configuracion debe tener el siguiente formato:
    /// {NOMBRE}={VALOR}
    /// Debe incluir todos los valores econtrados en la estructura Config.
    /// Devuelve CustomError si:
    /// - No se pudo encontrar el archivo.
    /// - El archivo tiene un formato invalido.
    /// - El archivo no contiene todos los valores requeridos.
    pub fn from_file(path: &str) -> Result<Self, CustomError> {
        let file = File::open(path).map_err(|_| CustomError::ConfigMissingFile)?;
        Self::from_reader(file)
    }

    /// Crea un config a partir de cualquier implementacion del trait Read
    /// con el contenido en el formato mencionado en la documentacion de from_file.
    /// Devuelve CustomError si:
    /// - El contenido tiene un formato invalido.
    /// - El contenido no contiene todos los valores requeridos.
    /// - No se pudo leer el contenido.
    fn from_reader<T: Read>(content: T) -> Result<Config, CustomError> {
        let reader = BufReader::new(content);

        let mut config = Self {
            seed: String::new(),
            protocol_version: 0,
            port: 0,
            log_file: String::new(),
            npeers: 0,
            client_only: false,
            store_path: String::from("store"),
        };

        for line in reader.lines() {
            let current_line = line.map_err(|_| CustomError::ConfigInvalid)?;

            let setting: Vec<&str> = current_line.split('=').collect();

            // ['KEY', 'VALUE'].len() == 2
            if setting.len() != 2 {
                return Err(CustomError::ConfigInvalid);
            }
            Self::load_setting(&mut config, setting[0], setting[1])?;
        }

        Self::check_required_values(&config)?;

        Ok(config)
    }

    /// Verifica que todos los valores requeridos esten cargados en el config.
    fn check_required_values(config: &Config) -> Result<(), CustomError> {
        if config.seed.is_empty() {
            return Err(CustomError::ConfigMissingValue);
        }
        if config.protocol_version == 0 {
            return Err(CustomError::ConfigMissingValue);
        }
        if config.port == 0 {
            return Err(CustomError::ConfigMissingValue);
        }
        if config.log_file.is_empty() {
            return Err(CustomError::ConfigMissingValue);
        }
        if config.npeers == 0 {
            return Err(CustomError::ConfigMissingValue);
        }
        Ok(())
    }

    /// Carga un "value" en el config en base al "name" que recibe.
    /// Devuelve CustomError si:
    /// - El "name" no es un nombre valido.
    /// - El "value" no se pudo convertir al tipo esperado.
    fn load_setting(&mut self, name: &str, value: &str) -> Result<(), CustomError> {
        match name {
            "SEED" => self.seed = String::from(value),
            "PROTOCOL_VERSION" => {
                self.protocol_version =
                    i32::from_str(value).map_err(|_| CustomError::ConfigErrorReadingValue)?
            }
            "PORT" => {
                self.port =
                    u16::from_str(value).map_err(|_| CustomError::ConfigErrorReadingValue)?
            }
            "LOG" => self.log_file = String::from(value),
            "NPEERS" => {
                self.npeers =
                    u8::from_str(value).map_err(|_| CustomError::ConfigErrorReadingValue)?
            }
            "STORE_PATH" => self.store_path = String::from(value),
            "CLIENT_ONLY" => self.client_only = value == "true",
            _ => (),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_con_formato_invalido() {
        let content = "KEY".as_bytes();
        let config = Config::from_reader(content);
        assert!(config.is_err());
        assert!(matches!(config, Err(CustomError::ConfigInvalid)));
    }

    #[test]
    fn config_con_valores_faltantes() {
        let content = "SEED=seed.test\n".as_bytes();
        let config = Config::from_reader(content);
        assert!(config.is_err());
        assert!(matches!(config, Err(CustomError::ConfigMissingValue)));
    }

    #[test]
    fn config_con_valor_vacio() {
        let content = "SEED=\n\
        PROTOCOL_VERSION=1234\n\
        LOG=log.txt\n\
        NPEERS=5\n\
        PORT=4321\n\
        CLIENT_ONLY=false\n\
        STORE_PATH=store"
            .as_bytes();
        let config = Config::from_reader(content);
        assert!(config.is_err());
        assert!(matches!(config, Err(CustomError::ConfigMissingValue)));
    }

    #[test]
    fn config_con_valores_requeridos() -> Result<(), CustomError> {
        let content = "SEED=seed.test\n\
        PROTOCOL_VERSION=7000\n\
        LOG=log.txt\n\
        NPEERS=5\n\
        PORT=4321\n\
        CLIENT_ONLY=true\n\
        STORE_PATH=custom"
            .as_bytes();
        let config = Config::from_reader(content)?;
        assert_eq!(7000, config.protocol_version);
        assert_eq!("seed.test", config.seed);
        assert_eq!(5, config.npeers);
        assert_eq!("log.txt", config.log_file);
        assert_eq!(4321, config.port);
        assert_eq!(true, config.client_only);
        assert_eq!("custom", config.store_path);

        let content = "SEED=seed.test\n\
        PROTOCOL_VERSION=7000\n\
        LOG=log.txt\n\
        NPEERS=5\n\
        PORT=4321"
            .as_bytes();
        let config = Config::from_reader(content)?;
        assert_eq!(7000, config.protocol_version);
        assert_eq!("seed.test", config.seed);
        assert_eq!(5, config.npeers);
        assert_eq!("log.txt", config.log_file);
        assert_eq!(4321, config.port);
        assert_eq!(false, config.client_only);
        assert_eq!("store", config.store_path);

        Ok(())
    }

    #[test]
    fn config_con_valores_de_mas() -> Result<(), CustomError> {
        let content = "SEED=seed.test\n\
        VALOR_NO_REQUERIDO=1234\n\
        PROTOCOL_VERSION=7000\n\
        LOG=log.txt\n\
        NPEERS=5\n\
        PORT=4321\n\
        CLIENT_ONLY=true\n\
        STORE_PATH=custom"
            .as_bytes();
        let config = Config::from_reader(content)?;
        assert_eq!(7000, config.protocol_version);
        assert_eq!("seed.test", config.seed);
        assert_eq!(5, config.npeers);
        assert_eq!("log.txt", config.log_file);
        assert_eq!(4321, config.port);
        assert_eq!(true, config.client_only);
        assert_eq!("custom", config.store_path);

        let content = "SEED=seed.test\n\
        VALOR_NO_REQUERIDO=\n\
        PROTOCOL_VERSION=7000\n\
        LOG=log.txt\n\
        NPEERS=5\n\
        PORT=4321\n\
        CLIENT_ONLY=true\n\
        STORE_PATH=custom"
            .as_bytes();
        let config = Config::from_reader(content)?;
        assert_eq!(7000, config.protocol_version);
        assert_eq!("seed.test", config.seed);
        assert_eq!(5, config.npeers);
        assert_eq!("log.txt", config.log_file);
        assert_eq!(4321, config.port);
        assert_eq!(true, config.client_only);
        assert_eq!("custom", config.store_path);
        Ok(())
    }
}
