//! REPL interactivo para insertar, eliminar y buscar llaves en un árbol B cuyo
//! orden se elige en runtime (3-8). Tras cada inserción o eliminación imprime el
//! árbol, y el comando `verificar` reporta invariantes y balanceo.

use bonsai_rs::{ArbolB, KVPair};
use std::io::{self, Write};

/// Adaptador para usar `ArbolB<(), M>` con `M` elegido en runtime,
/// borrando el const generic mediante `Box<dyn ArbolDyn>`.
trait ArbolDyn {
    fn insertar(&mut self, k: i32);
    fn elimina(&mut self, k: i32) -> Option<KVPair<()>>;
    fn busca(&self, k: i32) -> bool;
    fn a_cadena(&self) -> String;
    fn verificar(&self) -> String;
}

impl<const M: usize> ArbolDyn for ArbolB<(), M> {
    fn insertar(&mut self, k: i32) {
        self.insertar(k, ());
    }

    fn elimina(&mut self, k: i32) -> Option<KVPair<()>> {
        self.elimina(k)
    }

    fn busca(&self, k: i32) -> bool {
        self.busca(k).is_some()
    }

    fn a_cadena(&self) -> String {
        self.a_cadena_claves()
    }

    fn verificar(&self) -> String {
        self.verificar()
    }
}

/// Construye un `Box<dyn ArbolDyn>` del orden pedido (3-8), o `None` si no está soportado.
fn nuevo_arbol(m: usize) -> Option<Box<dyn ArbolDyn>> {
    match m {
        3 => Some(Box::new(ArbolB::<(), 3>::new())),
        4 => Some(Box::new(ArbolB::<(), 4>::new())),
        5 => Some(Box::new(ArbolB::<(), 5>::new())),
        6 => Some(Box::new(ArbolB::<(), 6>::new())),
        7 => Some(Box::new(ArbolB::<(), 7>::new())),
        8 => Some(Box::new(ArbolB::<(), 8>::new())),
        _ => None,
    }
}

/// Parsea una llave `i32` desde la entrada.
fn parse_key(s: &str) -> Option<i32> {
    s.parse().ok()
}

/// Pide el orden del árbol, crea el REPL y procesa los comandos hasta `exit`.
fn main() {
    println!("--- Bonsai REPL ---");

    let orden = loop {
        print!("Ingresa el orden del árbol B (3-8): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            return;
        }

        match input.trim().parse::<usize>() {
            Ok(m) if m >= 3 && m <= 8 => break m,
            _ => println!("Orden inválido. Debe ser un entero entre 3 y 8."),
        }
    };

    let mut arbol = match nuevo_arbol(orden) {
        Some(a) => a,
        None => {
            println!("Orden no soportado: {orden}");
            return;
        }
    };

    println!("Árbol B de orden {orden} creado.");
    println!("Comandos: insertar <llave> | eliminar <llave> | buscar <llave> | verificar | exit");
    println!("{}", arbol.a_cadena());

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "insertar" => cmd_insertar(&mut *arbol, args),
            "eliminar" => cmd_eliminar(&mut *arbol, args),
            "buscar" => cmd_buscar(&*arbol, args),
            "verificar" => cmd_verificar(&*arbol),
            "exit" => {
                println!("Saliendo.");
                break;
            }
            _ => println!(
                "Comando no reconocido. Sintaxis: insertar <llave> | eliminar <llave> | buscar <llave> | exit"
            ),
        }
    }
}

/// Inserta una llave y reimprime el árbol.
fn cmd_insertar(arbol: &mut dyn ArbolDyn, args: &[&str]) {
    let [key_str] = args else {
        println!("Uso: insertar <llave>");
        return;
    };
    let Some(key) = parse_key(key_str) else {
        println!("Error: llave inválida");
        return;
    };

    arbol.insertar(key);
    println!("Llave {key} insertada.");
    println!("{}", arbol.a_cadena());
}

/// Elimina una llave y reimprime el árbol.
fn cmd_eliminar(arbol: &mut dyn ArbolDyn, args: &[&str]) {
    let [key_str] = args else {
        println!("Uso: eliminar <llave>");
        return;
    };
    let Some(key) = parse_key(key_str) else {
        println!("Error: llave inválida");
        return;
    };

    match arbol.elimina(key) {
        Some(_) => println!("Llave {key} eliminada."),
        None => println!("Llave {key} no encontrada."),
    }
    println!("{}", arbol.a_cadena());
}

/// Indica si una llave está en el árbol.
fn cmd_buscar(arbol: &dyn ArbolDyn, args: &[&str]) {
    let [key_str] = args else {
        println!("Uso: buscar <llave>");
        return;
    };
    let Some(key) = parse_key(key_str) else {
        println!("Error: llave inválida");
        return;
    };

    if arbol.busca(key) {
        println!("Llave {key} encontrada.");
    } else {
        println!("Llave {key} no encontrada.");
    }
}

/// Imprime el reporte de verificación del árbol.
fn cmd_verificar(arbol: &dyn ArbolDyn) {
    print!("{}", arbol.verificar());
}

