# bonsai-rs

Implementacion de árboles B.

## Contenido

La mayoría de los métodos están implementados para la estructura `Node`, pero la API pública se debe usar mediante la estructura `ArbolB`.
Esto es por que la raíz del árbol se comporta ligeramente distinto a los demás nodos, y la estructura `ArbolB` encapsula un `Node` en su único campo `root`, por lo que `ArbolB` es el responsable de manejar los casos especiales.

Dado que los árboles B se pueden usar también como estructuras para almacenar mapeos, la estructura `ArbolB` es genérica sobre un parámetro `T` para los valores asociados a las llaves. Las llaves siempre son `i32` aunque fácilmente se podría generalizar.

Además, se incopora un parámetro constante genérico `M` para el orden de los árboles, siendo el valor por defecto `M = 3`.

El hecho de que esta constante sea parte del tipo implica que es imposible mezclar árboles de diferente orden, y esto está garantizado en tiempo de compilación.

Se implementa también un REPL básico para insertar, buscar y eliminar llaves.

## Ejecutar las pruebas

```bash
cargo test
```

## Ejecutar el REPL

```bash
cargo run --bin repl
```

### Comandos disponibles

| Comando | Descripción |
|---------|-------------|
| `insertar <llave>` | Inserta una llave |
| `buscar <llave>` | Busca una llave |
| `eliminar <llave>` | Elimina una llave |
| `exit` | Sale del REPL |
