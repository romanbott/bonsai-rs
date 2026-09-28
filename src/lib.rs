//! Árbol B de orden configurable (`M` = máximo de hijos por nodo) sobre pares `i32 -> T`.
//!
//! El orden `M` es un parámetro constante de tipo (`M = 3` por defecto). Cada nodo
//! guarda llaves ordenadas y subárboles en los "huecos" entre ellas
//! (`hijos = llaves + 1`). La inserción y la eliminación son reactivas: propagan
//! hacia arriba el split (`Err` con la llave promovida) o el underflow
//! (`Err(Underflow)`) para que el llamador rebalancee.
//!
//! ```
//! use bonsai_rs::ArbolB;
//!
//! let mut arbol = ArbolB::<&str, 3>::new();
//! arbol.insertar(1, "uno");
//! arbol.insertar(2, "dos");
//!
//! assert!(arbol.busca(1).is_some());
//! assert_eq!(arbol.elimina(2).is_some(), true);
//! assert!(arbol.busca(2).is_none());
//! assert_eq!(arbol.a_cadena_claves(), "[1]");
//! ```

use std::fmt;
use std::mem;

/// Par llave-valor almacenado en un nodo (llave `i32`, valor `T`).
#[derive(PartialEq, Eq, Clone)]
pub struct KVPair<T: Clone> {
    key: i32,
    value: T,
}

/// Par de valor unitario a partir de una llave (para árboles de solo llaves).
impl From<i32> for KVPair<()> {
    fn from(value: i32) -> Self {
        KVPair {
            key: value,
            value: (),
        }
    }
}

/// Formatea como `llave: valor`.
impl<T: fmt::Debug + Clone> fmt::Debug for KVPair<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {:?}", self.key, self.value)
    }
}

/// Resultado de eliminar en un nodo.
///
/// - `NoEncontrada`: la llave no está en el subárbol.
/// - `Underflow`: se eliminó, pero el nodo quedó bajo el mínimo de llaves y el
///   llamador debe rebalancearlo (lleva el par eliminado).
#[derive(PartialEq, Eq, Debug)]
enum ElimError<T: Clone> {
    NoEncontrada,
    Underflow(KVPair<T>),
}

/// Nodo del árbol B: llaves ordenadas y subárboles en los huecos entre ellas.
/// Un nodo interno cumple `hijos = llaves + 1`.
struct Node<T: Clone, const M: usize = 3> {
    keys: Vec<KVPair<T>>,
    children: Vec<Node<T, M>>,
}

impl<T: Clone, const M: usize> Node<T, M> {
    /// Mínimo de llaves de un nodo no-raíz: `(M + 1) / 2 - 1`.
    const Q: usize = (M + 1) / 2 - 1;

    /// Índice del primer hueco a la derecha de `key` (posición de búsqueda/inserción).
    fn key_pos(&self, key: i32) -> usize {
        self.keys
            .iter()
            .position(|kv| kv.key > key)
            .unwrap_or(self.keys.len())
    }

    /// Hijo por el que descender para `key`.
    fn busca_hijo(&self, key: i32) -> &Node<T, M> {
        &self.children[self.key_pos(key)]
    }

    /// `true` si el nodo no tiene hijos.
    fn es_hoja(&self) -> bool {
        self.children.len() == 0
    }

    /// Busca `key` en el subárbol; devuelve el par si existe.
    fn busca(&self, key: i32) -> Option<&KVPair<T>> {
        if let Some(kv) = self.keys.iter().find(|kv| kv.key == key) {
            return Some(kv);
        }

        if self.es_hoja() {
            return None;
        }

        let hijo = self.busca_hijo(key);

        hijo.busca(key)
    }

    /// Inserta `kv` en el subárbol.
    ///
    /// Si el nodo desborda, devuelve `Err(kv)` con la llave a promover: el llamador
    /// debe hacer `split` e insertar la llave en el padre.
    fn insertar(&mut self, kv: KVPair<T>) -> Result<(), KVPair<T>> {
        if self.es_hoja() {
            let pos = self.key_pos(kv.key);

            self.keys.insert(pos, kv);

            if self.keys.len() < M {
                return Ok(());
            }

            return Err(self.keys.remove(M / 2));
        }

        let pos = self.key_pos(kv.key);

        let promovida = match self.children[pos].insertar(kv) {
            Ok(_) => {
                return Ok(());
            }
            Err(promovida) => promovida,
        };

        let node_to_split = self.children.remove(pos);

        let (left, right) = node_to_split.split();

        self.children.insert(pos, left);
        self.children.insert(pos + 1, right);

        self.keys.insert(pos, promovida);

        if self.keys.len() < M {
            return Ok(());
        }

        return Err(self.keys.remove(M / 2));
    }

    /// Divide el nodo desbordado en dos, repartiendo llaves e hijos
    /// (la izquierda queda con `M / 2` llaves).
    fn split(mut self) -> (Node<T, M>, Node<T, M>) {
        assert!(self.keys.len() == M - 1);

        if self.es_hoja() {
            let right_keys = self.keys.split_off(M / 2);

            let left = Node {
                keys: self.keys,
                children: vec![],
            };

            let right = Node {
                keys: right_keys,
                children: vec![],
            };
            return (left, right);
        }

        assert!(self.children.len() == M + 1);

        let right_children = self.children.split_off(M / 2 + 1);
        let right_keys = self.keys.split_off(M / 2);

        let left = Node {
            keys: self.keys,
            children: self.children,
        };

        let right = Node {
            keys: right_keys,
            children: right_children,
        };
        return (left, right);
    }

    /// Par de menor llave del subárbol.
    fn min(&self) -> &KVPair<T> {
        if self.children.len() == 0 {
            self.keys.first().unwrap()
        } else {
            self.children.first().unwrap().min()
        }
    }

    /// Par de mayor llave del subárbol.
    fn max(&self) -> &KVPair<T> {
        if self.children.len() == 0 {
            self.keys.last().unwrap()
        } else {
            self.children.last().unwrap().max()
        }
    }

    /// Elimina `key` del subárbol.
    ///
    /// Devuelve el par eliminado, `Underflow` si el nodo quedó bajo el mínimo
    /// (el llamador debe rebalancear), o `NoEncontrada` si la llave no está.
    fn elimina(&mut self, key: i32) -> Result<KVPair<T>, ElimError<T>> {
        // Caso base: nodo hoja
        if self.es_hoja() {
            return match self.keys.iter().position(|kv| kv.key == key) {
                // La llave no existe en el árbol
                None => Err(ElimError::NoEncontrada),
                Some(i) => {
                    let kv = self.keys.remove(i);

                    // Si el nodo quedó por debajo del mínimo de llaves, avisar al caller
                    if self.keys.len() < Self::Q {
                        Err(ElimError::Underflow(kv))
                    } else {
                        Ok(kv)
                    }
                }
            };
        }

        // Nodo interno: la llave a eliminar está en este nodo
        if let Some(i) = self.keys.iter().position(|kv| kv.key == key) {
            // Reemplazar la llave por su predecesor (máximo del subárbol izquierdo)
            let predecesor = self.children[i].max().clone();
            let pk = predecesor.key;
            let eliminada = mem::replace(&mut self.keys[i], predecesor);

            // Eliminar el predecesor (ahora duplicado) del subárbol izquierdo
            return match self.children[i].elimina(pk) {
                // Si el hijo quedó bajo el mínimo, el caller debe rebalancearlo
                Err(ElimError::Underflow(_)) => self.rebalancea(i, eliminada),
                // El resto de los casos no altera este nodo: ya se hizo el reemplazo
                _ => Ok(eliminada),
            };
        }

        // Nodo interno: la llave no está, bajar por el hijo que la contiene
        let pos = self.key_pos(key);

        match self.children[pos].elimina(key) {
            // Si el hijo quedó bajo el mínimo, el caller debe rebalancearlo
            Err(ElimError::Underflow(kv)) => self.rebalancea(pos, kv),
            // Propagar el resultado tal cual (encontrada/eliminada o no encontrada)
            otro => otro,
        }
    }

    /// Repara el underflow del hijo en `pos`: préstamo del hermano izquierdo o
    /// derecho, o fusión. Propaga `Underflow` si este nodo también queda bajo el mínimo.
    fn rebalancea(&mut self, pos: usize, kv: KVPair<T>) -> Result<KVPair<T>, ElimError<T>> {
        debug_assert!(self.children[pos].keys.len() == Self::Q - 1);

        if (pos > 0) && (self.children[pos - 1].keys.len() > Self::Q) {
            let key_from_sibling = self.children[pos - 1].keys.pop().unwrap();

            let parent = mem::replace(&mut self.keys[pos - 1], key_from_sibling);

            self.children.get_mut(pos).unwrap().keys.insert(0, parent);

            if let Some(node_from_sibling) = self.children[pos - 1].children.pop() {
                self.children
                    .get_mut(pos)
                    .unwrap()
                    .children
                    .insert(0, node_from_sibling);
            }

            debug_assert!(self.children[pos].keys.len() == Self::Q);
            debug_assert!(
                self.children[pos].es_hoja()
                    || self.children[pos].children.len() == self.children[pos].keys.len() + 1
            );

            return Ok(kv);
        }

        if (pos + 1 < self.children.len()) && (self.children[pos + 1].keys.len() > Self::Q) {
            let key_from_sibling = self.children[pos + 1].keys.remove(0);

            let parent = mem::replace(&mut self.keys[pos], key_from_sibling);

            self.children.get_mut(pos).unwrap().keys.push(parent);

            if !self.children[pos + 1].children.is_empty() {
                let node_from_sibling = self.children[pos + 1].children.remove(0);

                self.children
                    .get_mut(pos)
                    .unwrap()
                    .children
                    .push(node_from_sibling);
            }

            debug_assert!(self.children[pos].keys.len() == Self::Q);
            debug_assert!(
                self.children[pos].es_hoja()
                    || self.children[pos].children.len() == self.children[pos].keys.len() + 1
            );

            return Ok(kv);
        }

        let izq = pos.saturating_sub(1);
        let der = izq + 1;

        if der < self.children.len() {
            let separador = self.keys.remove(izq);

            let Node { keys, children } = self.children.remove(der);

            self.children[izq].keys.push(separador);
            self.children[izq].keys.extend(keys.into_iter());

            self.children[izq].children.extend(children.into_iter());

            debug_assert!(self.children[izq].keys.len() <= M - 1);
            debug_assert!(
                self.children[izq].es_hoja()
                    || self.children[izq].children.len() == self.children[izq].keys.len() + 1
            );

            if self.keys.len() < Self::Q {
                return Err(ElimError::Underflow(kv));
            } else {
                return Ok(kv);
            }
        }

        unreachable!("rebalancea: no hay hermano para reparar el underflow")
    }
}

impl<T: Clone, const M: usize> Node<T, M> {
    /// Cotas (llaves vecinas) del hueco `i`, para etiquetar el rango del hijo.
    fn hueco(&self, i: usize) -> (Option<i32>, Option<i32>) {
        let lo = if i > 0 {
            self.keys.get(i - 1).map(|kv| kv.key)
        } else {
            None
        };
        let hi = self.keys.get(i).map(|kv| kv.key);
        (lo, hi)
    }

    /// Representa las llaves del nodo como `[k0, k1, ...]` usando el formateador dado.
    fn cabecera_con(&self, f: &dyn Fn(&KVPair<T>) -> String) -> String {
        let ks = self.keys.iter().map(f).collect::<Vec<_>>().join(", ");
        format!("[{ks}]")
    }

    /// Etiqueta de un hijo: su rango de hueco más la cabecera del hijo.
    fn etiqueta_hijo_con(
        &self,
        lo: Option<i32>,
        hi: Option<i32>,
        f: &dyn Fn(&KVPair<T>) -> String,
    ) -> String {
        let rango = match (lo, hi) {
            (None, None) => String::new(),
            (None, Some(h)) => format!("(<{h})"),
            (Some(l), None) => format!("(>{l})"),
            (Some(l), Some(h)) => format!("({l}..{h})"),
        };
        format!("{rango:<10}{}", self.cabecera_con(f))
    }

    /// Dibuja recursivamente el subárbol con conectores de árbol.
    fn imprime_hijos_con(&self, buf: &mut String, prefijo: &str, f: &dyn Fn(&KVPair<T>) -> String) {
        let n = self.children.len();
        for (i, hijo) in self.children.iter().enumerate() {
            let ultimo = i + 1 == n;
            let conector = if ultimo { "└── " } else { "├── " };

            let (lo, hi) = self.hueco(i);
            buf.push_str(&format!(
                "{prefijo}{conector}{}\n",
                hijo.etiqueta_hijo_con(lo, hi, f)
            ));

            let pref = format!("{prefijo}{}", if ultimo { "    " } else { "│   " });
            hijo.imprime_hijos_con(buf, &pref, f);
        }
    }

    /// Recorre el subárbol verificando las cotas de llaves e hijos, y acumula
    /// las profundidades de las hojas.
    fn verifica_nodo(&self, prof: usize, es_raiz: bool, buf: &mut String, hojas: &mut Vec<usize>) {
        let (min_keys, max_keys) = if es_raiz {
            (0, M - 1)
        } else {
            (Self::Q, M - 1)
        };
        let llaves_ok = self.keys.len() >= min_keys && self.keys.len() <= max_keys;
        let hijos_ok = self.es_hoja() || self.children.len() == self.keys.len() + 1;

        let cabecera = self.cabecera_con(&|kv| kv.key.to_string());
        buf.push_str(&format!(
            "{cabecera}: {} llaves, {} hijos",
            self.keys.len(),
            self.children.len()
        ));
        if !llaves_ok {
            buf.push_str(&format!("  [INCORRECTO] llaves fuera de [{min_keys}, {max_keys}]"));
        }
        if !hijos_ok {
            buf.push_str("  [INCORRECTO] hijos != llaves + 1");
        }
        if llaves_ok && hijos_ok {
            buf.push_str("  [CORRECTO]");
        }
        buf.push('\n');

        if self.es_hoja() {
            hojas.push(prof);
        } else {
            for hijo in &self.children {
                hijo.verifica_nodo(prof + 1, false, buf, hojas);
            }
        }
    }
}

/// Representa el subárbol en formato de árbol con intervalos por hueco.
impl<T: fmt::Debug + Clone, const M: usize> fmt::Debug for Node<T, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buf = format!("{}\n", self.cabecera_con(&|kv| format!("{kv:?}")));
        self.imprime_hijos_con(&mut buf, "", &|kv| format!("{kv:?}"));
        write!(f, "{}", buf.trim_end())
    }
}

/// Árbol B de orden `M` (`M >= 3`). Gestiona el crecimiento y decrecimiento de la raíz.
pub struct ArbolB<T: Clone, const M: usize = 3> {
    root: Node<T, M>,
}

/// Atajo para un árbol de solo llaves (`ArbolB<()>`).
pub type ConjuntoB = ArbolB<()>;

/// Representa el árbol con intervalos por hueco (llave y valor).
impl<T: fmt::Debug + Clone, const M: usize> fmt::Debug for ArbolB<T, M> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.root)
    }
}

impl<T: fmt::Debug + Clone, const M: usize> ArbolB<T, M> {
    /// Devuelve la representación del árbol con llave y valor.
    pub fn a_cadena(&self) -> String {
        format!("{:?}", self)
    }

    /// Imprime el árbol (llave y valor) por `stdout`.
    pub fn imprime(&self) {
        println!("{:?}", self);
    }
}

impl<T: Clone, const M: usize> ArbolB<T, M> {
    /// Crea un árbol vacío.
    pub fn new() -> Self {
        assert!(M >= 3);
        Self {
            root: Node {
                keys: vec![],
                children: vec![],
            },
        }
    }

    /// Inserta (o reemplaza) el par `(clave, valor)`.
    pub fn insertar(&mut self, clave: i32, valor: T) {
        let kv = KVPair {
            key: clave,
            value: valor,
        };

        let promovida = match self.root.insertar(kv) {
            Ok(_) => {
                return;
            }
            Err(promovida) => promovida,
        };

        let new_root = Node {
            keys: vec![promovida],
            children: vec![],
        };

        let old_root = mem::replace(&mut self.root, new_root);

        let (left, right) = old_root.split();

        self.root.children.push(left);
        self.root.children.push(right);
    }

    /// `true` si todo nodo interno cumple `hijos = llaves + 1`.
    fn es_valido(&self) -> bool {
        fn check<T: Clone, const M: usize>(nodo: &Node<T, M>) -> bool {
            if !nodo.es_hoja() && nodo.children.len() != nodo.keys.len() + 1 {
                return false;
            }
            nodo.children.iter().all(check)
        }
        check(&self.root)
    }

    /// `true` si todas las hojas están a la misma profundidad.
    fn hojas_balanceadas(&self) -> bool {
        fn profundidades<T: Clone, const M: usize>(nodo: &Node<T, M>, prof: usize) -> Vec<usize> {
            if nodo.es_hoja() {
                vec![prof]
            } else {
                nodo.children
                    .iter()
                    .flat_map(|h| profundidades(h, prof + 1))
                    .collect()
            }
        }
        let profs = profundidades(&self.root, 0);
        profs.iter().all(|&p| p == profs[0])
    }

    /// Devuelve el par con `key`, si existe.
    pub fn busca(&self, key: i32) -> Option<&KVPair<T>> {
        self.root.busca(key)
    }

    /// Elimina `key`; devuelve el par eliminado o `None` si no estaba.
    pub fn elimina(&mut self, key: i32) -> Option<KVPair<T>> {
        let kv = match self.root.elimina(key) {
            Err(ElimError::NoEncontrada) => return None,
            Ok(kv) | Err(ElimError::Underflow(kv)) => kv,
        };

        if self.root.keys.is_empty() && self.root.children.len() == 1 {
            self.root = self.root.children.remove(0);
        }

        Some(kv)
    }

    /// Devuelve la representación del árbol mostrando solo las llaves.
    pub fn a_cadena_claves(&self) -> String {
        let mut buf = format!("{}\n", self.root.cabecera_con(&|kv| kv.key.to_string()));
        self.root
            .imprime_hijos_con(&mut buf, "", &|kv| kv.key.to_string());
        buf.trim_end().to_string()
    }

    /// Recorre el árbol y devuelve un reporte de verificación: por cada nodo sus
    /// cotas de llaves e hijos, más las profundidades de las hojas y el veredicto
    /// de balanceo.
    pub fn verificar(&self) -> String {
        let mut buf = String::new();
        let mut hojas = Vec::new();
        self.root.verifica_nodo(0, true, &mut buf, &mut hojas);

        buf.push_str(&format!("Hojas a profundidad: {hojas:?}\n"));
        let balanceadas = hojas.iter().all(|&p| p == hojas[0]);
        buf.push_str(if balanceadas {
            "Balanceado\n"
        } else {
            "DESBALANCEADO\n"
        });
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type Nodo<T> = Node<T, 3>;
    type Arbol3<T> = ArbolB<T, 3>;

    #[test]
    fn busca_simple() {
        let arbol = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert!(arbol.busca(1).is_some())
    }

    #[test]
    fn busca_simple_no_existe() {
        let arbol = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert!(arbol.busca(4).is_none())
    }

    #[test]
    fn busca_anidado() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into(), 8.into()],
            children: vec![],
        };

        let arbol = Nodo {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert!(arbol.busca(1).is_some());
        assert!(arbol.busca(4).is_some());
        assert!(arbol.busca(6).is_some());
        assert!(arbol.busca(5).is_none());
    }

    #[test]
    fn inserta_en_hoja() {
        let mut hoja = Nodo {
            keys: vec![1.into()],
            children: vec![],
        };

        assert!(hoja.insertar(2.into()).is_ok());
    }

    #[test]
    fn inserta_en_hoja_llena() {
        let mut hoja = Nodo {
            keys: vec![1.into(), 3.into()],
            children: vec![],
        };

        assert!(hoja.insertar(2.into()).is_err());

        assert_eq!(hoja.keys, vec![1.into(), 3.into()])
    }

    #[test]
    fn inserta_en_nodo_interno() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let mut arbol = Nodo {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert!(arbol.insertar(7.into()).is_ok())
    }

    #[test]
    fn inserta_con_split_simple() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let mut arbol = Nodo {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert!(arbol.insertar(3.into()).is_ok());
        assert!(arbol.keys.len() == 2);
        assert!(arbol.children.len() == 3);
    }

    #[test]
    fn inserta_con_split_nodo_interno() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let hoja3 = Nodo {
            keys: vec![12.into()],
            children: vec![],
        };

        let mut arbol = Nodo {
            keys: vec![4.into(), 10.into()],
            children: vec![hoja1, hoja2, hoja3],
        };

        assert_eq!(arbol.insertar(3.into()), Err(4.into()));
        assert!(arbol.keys.len() == 2);
        assert!(arbol.children.len() == 4);
    }

    #[test]
    fn inserta_en_arbol_con_split_nodo_interno() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let hoja3 = Nodo {
            keys: vec![12.into()],
            children: vec![],
        };

        let mut arbol = Arbol3 {
            root: Nodo {
                keys: vec![4.into(), 10.into()],
                children: vec![hoja1, hoja2, hoja3],
            },
        };

        arbol.insertar(3, ());
        assert!(arbol.root.keys.len() == 1);
        assert!(arbol.root.children.len() == 2);
    }

    #[test]
    fn imprime_con_intervalos() {
        let kv = |k: i32, v: char| KVPair { key: k, value: v };

        let hoja15 = Nodo {
            keys: vec![kv(15, 'c')],
            children: vec![],
        };
        let hoja25 = Nodo {
            keys: vec![kv(25, 'd')],
            children: vec![],
        };
        let nodo20 = Nodo {
            keys: vec![kv(20, 'b')],
            children: vec![hoja15, hoja25],
        };
        let hoja5 = Nodo {
            keys: vec![kv(5, 'a')],
            children: vec![],
        };
        let hoja40 = Nodo {
            keys: vec![kv(40, 'e')],
            children: vec![],
        };

        let arbol = Arbol3 {
            root: Nodo {
                keys: vec![kv(10, 'A'), kv(30, 'B')],
                children: vec![hoja5, nodo20, hoja40],
            },
        };

        let esperado = concat!(
            "[10: 'A', 30: 'B']\n",
            "├── (<10)     [5: 'a']\n",
            "├── (10..30)  [20: 'b']\n",
            "│   ├── (<20)     [15: 'c']\n",
            "│   └── (>20)     [25: 'd']\n",
            "└── (>30)     [40: 'e']",
        );

        assert_eq!(arbol.a_cadena(), esperado);
    }

    #[test]
    fn imprime_arbol_tras_insertar_no_panics() {
        let mut arbol = Arbol3::new();

        for k in [5, 3, 8, 1, 4, 7, 9, 2, 6, 0] {
            arbol.insertar(k, 'x');
            arbol.imprime();
        }
    }

    #[test]
    fn invariante_arbol_tras_insertar_muchos() {
        let mut arbol = Arbol3::new();

        for k in 0..=30 {
            arbol.insertar(k, 'x');
        }

        for k in 0..=30 {
            assert!(arbol.root.busca(k).is_some(), "llave {k} no encontrada");
        }
        assert!(arbol.root.busca(99).is_none());
        assert!(arbol.es_valido(), "invariante hijos = llaves + 1 violada");
        assert!(arbol.hojas_balanceadas(), "hojas a distinta profundidad");
    }

    #[test]
    fn split_m4_promueve_la_tercera_llave() {
        let mut arbol = ArbolB::<char, 4>::new();

        for k in [1, 2, 3, 4] {
            arbol.insertar(k, 'x');
        }

        assert_eq!(arbol.root.keys.len(), 1);
        assert_eq!(arbol.root.keys[0].key, 3);
        assert_eq!(arbol.root.children.len(), 2);
        assert_eq!(
            arbol.root.children[0].keys,
            vec![KVPair { key: 1, value: 'x' }, KVPair { key: 2, value: 'x' }]
        );
        assert_eq!(
            arbol.root.children[1].keys,
            vec![KVPair { key: 4, value: 'x' }]
        );
    }

    fn insertar_y_verificar<const M: usize>(n: i32) {
        let mut arbol = ArbolB::<char, M>::new();

        for k in 0..=n {
            arbol.insertar(k, 'x');
        }

        for k in 0..=n {
            assert!(arbol.busca(k).is_some(), "M={M}: llave {k} no encontrada");
        }
        assert!(arbol.busca(n + 1).is_none(), "M={M}: sobra una llave");
        assert!(
            arbol.es_valido(),
            "M={M}: se rompe invariante hijos = llaves + 1"
        );
        assert!(
            arbol.hojas_balanceadas(),
            "M={M}: hojas a distinta profundidad"
        );
    }

    #[test]
    fn invariantes_con_distintos_m() {
        insertar_y_verificar::<3>(30);
        insertar_y_verificar::<4>(60);
        insertar_y_verificar::<5>(60);
        insertar_y_verificar::<6>(60);
    }

    #[test]
    fn elimina_de_hoja_sin_underflow() {
        let mut hoja = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert_eq!(hoja.elimina(1), Ok(1.into()));
        assert_eq!(hoja.keys, vec![2.into()]);
    }

    #[test]
    fn elimina_de_hoja_retorna_underflow() {
        let mut hoja = Nodo {
            keys: vec![1.into()],
            children: vec![],
        };

        assert_eq!(hoja.elimina(1), Err(ElimError::Underflow(1.into())));
        assert!(hoja.keys.is_empty());
    }

    #[test]
    fn elimina_no_encontrada() {
        let mut hoja = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };

        assert_eq!(hoja.elimina(5), Err(ElimError::NoEncontrada));
        assert_eq!(hoja.keys, vec![1.into(), 2.into()]);
    }

    #[test]
    fn elimina_de_nodo_interno() {
        let hoja1 = Nodo {
            keys: vec![1.into(), 2.into()],
            children: vec![],
        };
        let hoja2 = Nodo {
            keys: vec![6.into()],
            children: vec![],
        };

        let mut arbol = Nodo {
            keys: vec![4.into()],
            children: vec![hoja1, hoja2],
        };

        assert_eq!(arbol.elimina(4), Ok(4.into()));
        assert_eq!(arbol.keys, vec![2.into()]);
        assert_eq!(arbol.children[0].keys, vec![1.into()]);
        assert_eq!(arbol.children[1].keys, vec![6.into()]);
    }

    #[test]
    fn elimina_desde_arbol() {
        let mut arbol = Arbol3::new();
        arbol.insertar(1, 'x');
        arbol.insertar(2, 'x');

        assert_eq!(arbol.elimina(99), None);

        assert_eq!(arbol.elimina(1), Some(KVPair { key: 1, value: 'x' }));
        assert!(arbol.busca(1).is_none());
        assert!(arbol.busca(2).is_some());

        assert_eq!(arbol.elimina(2), Some(KVPair { key: 2, value: 'x' }));
        assert!(arbol.busca(2).is_none());
        assert!(arbol.es_valido());
    }

    #[test]
    fn elimina_secuencia_mantiene_invariantes() {
        let mut arbol = Arbol3::new();

        for k in 0..=20 {
            arbol.insertar(k, 'x');
        }

        for k in 0..=20 {
            arbol.elimina(k);
            assert!(arbol.busca(k).is_none(), "llave {k} sigue presente");
            assert!(arbol.es_valido(), "invariante violada tras eliminar {k}");
            assert!(
                arbol.hojas_balanceadas(),
                "hojas desbalanceadas tras eliminar {k}"
            );
        }

        assert!(arbol.es_valido());
        assert!(arbol.hojas_balanceadas());
    }

    fn eliminar_y_verificar<const M: usize>(n: i32) {
        let mut arbol = ArbolB::<char, M>::new();

        for k in 0..=n {
            arbol.insertar(k, 'x');
        }

        for k in 0..=n {
            arbol.elimina(k);
            assert!(arbol.busca(k).is_none(), "M={M}: llave {k} sigue presente");
            assert!(
                arbol.es_valido(),
                "M={M}: invariante violada tras eliminar {k}"
            );
            assert!(
                arbol.hojas_balanceadas(),
                "M={M}: hojas desbalanceadas tras eliminar {k}"
            );
        }

        assert!(arbol.es_valido());
        assert!(arbol.hojas_balanceadas());
    }

    #[test]
    fn elimina_con_distintos_m() {
        eliminar_y_verificar::<3>(30);
        eliminar_y_verificar::<4>(60);
        eliminar_y_verificar::<5>(60);
    }

    #[test]
    fn imprime_arbol_con_valor_unitario() {
        let mut arbol = ArbolB::<(), 4>::new();

        for k in [2, 1, 3, 4] {
            arbol.insertar(k, ());
        }

        let esperado = concat!(
            "[3: ()]\n",
            "├── (<3)      [1: (), 2: ()]\n",
            "└── (>3)      [4: ()]",
        );

        assert_eq!(arbol.a_cadena(), esperado);
    }

    #[test]
    fn rebalancea_prestamo_izquierdo_hojas() {
        let mut arbol = Nodo {
            keys: vec![10.into()],
            children: vec![
                Nodo {
                    keys: vec![1.into(), 5.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![15.into()],
                    children: vec![],
                },
            ],
        };

        let res = arbol.elimina(15);
        assert_eq!(res, Ok(15.into()));

        assert_eq!(arbol.keys, vec![5.into()]);
        assert_eq!(arbol.children[0].keys, vec![1.into()]);
        assert_eq!(arbol.children[1].keys, vec![10.into()]);
    }

    #[test]
    fn rebalancea_prestamo_izquierdo_nodos_internos() {
        let mut arbol = Nodo {
            keys: vec![20.into()],
            children: vec![
                Nodo {
                    keys: vec![8.into(), 15.into()],
                    children: vec![
                        Nodo {
                            keys: vec![5.into()],
                            children: vec![],
                        },
                        Nodo {
                            keys: vec![10.into()],
                            children: vec![],
                        },
                        Nodo {
                            keys: vec![18.into()],
                            children: vec![],
                        },
                    ],
                },
                Nodo {
                    keys: vec![],
                    children: vec![Nodo {
                        keys: vec![22.into()],
                        children: vec![],
                    }],
                },
            ],
        };

        let res = arbol.rebalancea(1, 25.into());

        assert_eq!(res, Ok(25.into()));

        assert_eq!(arbol.keys, vec![15.into()]);

        assert_eq!(arbol.children[0].keys, vec![8.into()]);
        assert_eq!(arbol.children[0].children.len(), 2);

        assert_eq!(arbol.children[1].keys, vec![20.into()]);
        assert_eq!(arbol.children[1].children.len(), 2);
        assert_eq!(arbol.children[1].children[0].keys, vec![18.into()]);
    }

    #[test]
    fn rebalancea_prestamo_derecho_hojas() {
        let mut arbol = Nodo {
            keys: vec![10.into()],
            children: vec![
                Nodo {
                    keys: vec![1.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![20.into(), 25.into()],
                    children: vec![],
                },
            ],
        };

        let res = arbol.elimina(1);
        assert_eq!(res, Ok(1.into()));

        assert_eq!(arbol.keys, vec![20.into()]);
        assert_eq!(arbol.children[0].keys, vec![10.into()]);
        assert_eq!(arbol.children[1].keys, vec![25.into()]);
    }

    #[test]
    fn rebalancea_prestamo_derecho_nodos_internos() {
        let mut arbol = Nodo {
            keys: vec![20.into()],
            children: vec![
                Nodo {
                    keys: vec![],
                    children: vec![Nodo {
                        keys: vec![10.into()],
                        children: vec![],
                    }],
                },
                Nodo {
                    keys: vec![30.into(), 40.into()],
                    children: vec![
                        Nodo {
                            keys: vec![25.into()],
                            children: vec![],
                        },
                        Nodo {
                            keys: vec![35.into()],
                            children: vec![],
                        },
                        Nodo {
                            keys: vec![45.into()],
                            children: vec![],
                        },
                    ],
                },
            ],
        };

        let res = arbol.rebalancea(0, 5.into());
        assert_eq!(res, Ok(5.into()));

        assert_eq!(arbol.keys, vec![30.into()]);

        assert_eq!(arbol.children[0].keys, vec![20.into()]);
        assert_eq!(arbol.children[0].children.len(), 2);
        assert_eq!(arbol.children[0].children[1].keys, vec![25.into()]);

        assert_eq!(arbol.children[1].keys, vec![40.into()]);
        assert_eq!(arbol.children[1].children.len(), 2);
    }

    #[test]
    fn rebalancea_fusion_izquierda() {
        let mut arbol = Nodo {
            keys: vec![10.into(), 30.into()],
            children: vec![
                Nodo {
                    keys: vec![5.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![20.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![40.into()],
                    children: vec![],
                },
            ],
        };

        let res = arbol.elimina(20);
        assert_eq!(res, Ok(20.into()));

        assert_eq!(arbol.keys, vec![30.into()]);
        assert_eq!(arbol.children.len(), 2);
        assert_eq!(arbol.children[0].keys, vec![5.into(), 10.into()]);
        assert_eq!(arbol.children[1].keys, vec![40.into()]);
    }

    #[test]
    fn rebalancea_fusion_derecha() {
        let mut arbol = Nodo {
            keys: vec![10.into(), 30.into()],
            children: vec![
                Nodo {
                    keys: vec![1.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![20.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![40.into()],
                    children: vec![],
                },
            ],
        };

        let res = arbol.elimina(1);
        assert_eq!(res, Ok(1.into()));

        assert_eq!(arbol.keys, vec![30.into()]);
        assert_eq!(arbol.children.len(), 2);
        assert_eq!(arbol.children[0].keys, vec![10.into(), 20.into()]);
        assert_eq!(arbol.children[1].keys, vec![40.into()]);
    }

    #[test]
    fn rebalancea_fusion_propaga_underflow() {
        let mut arbol = Nodo {
            keys: vec![10.into()],
            children: vec![
                Nodo {
                    keys: vec![1.into()],
                    children: vec![],
                },
                Nodo {
                    keys: vec![20.into()],
                    children: vec![],
                },
            ],
        };

        let res = arbol.elimina(1);
        assert_eq!(res, Err(ElimError::Underflow(1.into())));

        assert!(arbol.keys.is_empty());
        assert_eq!(arbol.children.len(), 1);
        assert_eq!(arbol.children[0].keys, vec![10.into(), 20.into()]);
    }

    #[test]
    fn rebalancea_fusion_derecha_nodos_internos() {
        let mut arbol = Nodo {
            keys: vec![20.into()],
            children: vec![
                Nodo {
                    keys: vec![],
                    children: vec![Nodo {
                        keys: vec![10.into()],
                        children: vec![],
                    }],
                },
                Nodo {
                    keys: vec![30.into()],
                    children: vec![
                        Nodo {
                            keys: vec![25.into()],
                            children: vec![],
                        },
                        Nodo {
                            keys: vec![35.into()],
                            children: vec![],
                        },
                    ],
                },
            ],
        };

        let res = arbol.rebalancea(0, 5.into());
        assert_eq!(res, Err(ElimError::Underflow(5.into())));

        assert!(arbol.keys.is_empty());
        assert_eq!(arbol.children.len(), 1);

        let fusionado = &arbol.children[0];
        assert_eq!(fusionado.keys, vec![20.into(), 30.into()]);
        assert_eq!(fusionado.children.len(), 3);
        assert_eq!(fusionado.children[0].keys, vec![10.into()]);
    }

    #[test]
    fn verificar_arbol_valido() {
        let mut arbol = ArbolB::<(), 4>::new();

        for k in [5, 3, 8, 1] {
            arbol.insertar(k, ());
        }

        let esperado = concat!(
            "[5]: 1 llaves, 2 hijos  [CORRECTO]\n",
            "[1, 3]: 2 llaves, 0 hijos  [CORRECTO]\n",
            "[8]: 1 llaves, 0 hijos  [CORRECTO]\n",
            "Hojas a profundidad: [1, 1]\n",
            "Balanceado\n",
        );

        assert_eq!(arbol.verificar(), esperado);
    }

    #[test]
    fn verificar_falla_llaves_excesivas() {
        let arbol = ArbolB {
            root: Nodo {
                keys: vec![1.into(), 2.into(), 3.into()],
                children: vec![],
            },
        };

        let reporte = arbol.verificar();
        assert!(reporte.contains("llaves fuera de [0, 2]"));
    }

    #[test]
    fn verificar_falla_llaves_insuficientes() {
        let arbol = ArbolB {
            root: Nodo {
                keys: vec![5.into()],
                children: vec![
                    Nodo {
                        keys: vec![],
                        children: vec![],
                    },
                    Nodo {
                        keys: vec![6.into()],
                        children: vec![],
                    },
                ],
            },
        };

        let reporte = arbol.verificar();
        assert!(reporte.contains("llaves fuera de [1, 2]"));
    }

    #[test]
    fn verificar_falla_hijos_incorrectos() {
        let arbol = ArbolB {
            root: Nodo {
                keys: vec![5.into()],
                children: vec![Nodo {
                    keys: vec![1.into()],
                    children: vec![],
                }],
            },
        };

        let reporte = arbol.verificar();
        assert!(reporte.contains("hijos != llaves + 1"));
    }

    #[test]
    fn verificar_falla_desbalanceado() {
        let arbol = ArbolB {
            root: Nodo {
                keys: vec![10.into(), 30.into()],
                children: vec![
                    Nodo {
                        keys: vec![5.into()],
                        children: vec![],
                    },
                    Nodo {
                        keys: vec![20.into()],
                        children: vec![
                            Nodo {
                                keys: vec![15.into()],
                                children: vec![],
                            },
                            Nodo {
                                keys: vec![25.into()],
                                children: vec![],
                            },
                        ],
                    },
                    Nodo {
                        keys: vec![40.into()],
                        children: vec![],
                    },
                ],
            },
        };

        let reporte = arbol.verificar();
        assert!(reporte.contains("Hojas a profundidad: [1, 2, 2, 1]"));
        assert!(reporte.contains("DESBALANCEADO"));
    }
}
