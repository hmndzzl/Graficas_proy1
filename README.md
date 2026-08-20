# SAT Runner 3D

## Características y Funcionalidades
El juego es un motor de renderizado pseudo-3D desarrollado en Rust, e incluye:
- **Renderizado 3D (Raycasting)**: Motor que implementa el algoritmo de raycasting ("método de estacas") para calcular la distancia a los muros y renderizarlos escalados verticalmente.
- **Corrección de "Fish Eye"**: Compensación trigonométrica para evitar la distorsión de "ojo de pez" que se produce normalmente al proyectar los rayos.
- **Renderizado 2D (Minimapa y Modo Top-Down)**: Permite visualizar todo el laberinto desde arriba (Minimapa HUD) o cambiar la cámara principal a un modo 2D superior.
- **Z-Buffer y Billboarding para Sprites**: Sistema matemático para que los sprites enemigos (modelos 2D) siempre miren hacia la cámara sin importar tu posición. Se dibujan utilizando su distancia a la cámara y un Z-Buffer 1D para evitar que atraviesen muros cercanos.
- **Colisiones (AABB)**: Sistema de colisiones integrado para que el jugador y los enemigos no atraviesen los muros del laberinto.
- **Animaciones por Cuadros (Spritesheets)**: Capacidad de recorrer y recortar un archivo (UV Mapping dinámico) para generar animaciones como el "screamer" en su persecución.
- **Audio Dinámico y Multipista**: Utilizando `rodio` se implementó BGM escalonado por niveles y sonido 3D para los enemigos (volumen variable según la cercanía al jugador).
- **Múltiples Niveles y Progresión**: Carga de diferentes diseños de laberintos.

## Librerías Utilizadas

| Librería | Descripción e Importancia |
| :--- | :--- |
| `minifb` | Crítica para el renderizado crudo del juego. Proporciona la creación de la ventana, entrada por teclado/mouse, y pinta directamente nuestro arreglo de píxeles (`Framebuffer`) a la pantalla. |
| `nalgebra-glm` | Facilita cálculos vectoriales como `Vec2` e interpolaciones matemáticas para la física y raycasting. |
| `image` | Carga de forma eficiente todos los recursos gráficos y texturas PNG/JPG, descodificando los píxeles para el uso del motor de render. |
| `rodio` | Sistema de manipulación asincrónica de audio de alto rendimiento, que permite ajustar sobre la marcha el nivel del volumen y reproducir distintas pistas (BGM, pasos, alertas) de manera independiente. |

## Controles

| Acción | Tecla / Ratón |
| :--- | :--- |
| **Mover Adelante / Atrás** | `W` / `S`  o  `↑` / `↓` (Flechas) |
| **Girar Izquierda / Derecha** | `A` / `D`  o  `←` / `→` (Flechas) |
| **Girar (Alternativo)** | Mover el **Ratón (Mouse)** de forma horizontal |
| **Pausa** | `P` |
| **Modo 2D / 3D** | `M` (Mientras se está jugando) |
| **Seleccionar / Empezar** | `ENTER` (En el Menú) |
| **Menú de Controles** | `C` (En el Menú) |
| **Reiniciar Nivel 1** | `R` (Al Perder / Ganar) |
| **Volver al Menú Principal**| `M` (Al Perder / Ganar) |
| **Salir** | `ESC` |

## Demostración (Video)

<img width="1363" height="929" alt="image" src="https://github.com/user-attachments/assets/1953502c-3e84-492f-a4ff-7c745dbc03a4" />

Link de video: https://youtu.be/YzSUZzLeVt0 
<br>
<br>

## Guía de Ejecución

**Prerrequisitos:**
Asegúrate de contar con la cadena de herramientas de **[Rust (cargo)](https://rustup.rs/)** instalada en tu sistema.

1. Clona o descarga el repositorio en tu máquina.

```bash
git clone https://github.com/hmndzzl/Graficas_proy1.git 
```

2. Abre tu terminal y navega hasta el directorio raíz del proyecto (`Graficas_proy1`), el cual debe contener el archivo `Cargo.toml`.

```bash
cd Graficas_proy1
```

3. Ejecuta el juego. Debido a que es un motor de renderizado de software (en CPU), el proyecto ha sido configurado en `Cargo.toml` para optimizarse fuertemente en su perfil de desarrollo por defecto. Usa simplemente:

```bash
cargo run
```

*Si necesitas un rendimiento más severo, también puedes correr `cargo run --release`.*

## Estructura del Proyecto

```text
Graficas_proy1/
├── assets/         # Carpeta con texturas (.png, .jpg) y audios (.mp3)
├── src/
│   ├── main.rs         # Ciclo del juego (Game Loop), control de estados, render2d y render3d.
│   ├── caster.rs       # Raycasting algorítmico, DDA e intersección con muros.
│   ├── enemy.rs        # Estructura e IA del enemigo (persecución), animaciones, audio dinámico.
│   ├── framebuffer.rs  # Abstracción para el dibujo de píxeles puros a nivel de buffer.
│   ├── maze.rs         # Lógica para leer y gestionar el layout en crudo (mapas .txt).
│   ├── physics.rs      # Colisiones matemáticas contra paredes y entre entidades.
│   ├── player.rs       # Entidad del jugador, manejo de input, movimiento.
│   ├── texture.rs      # Adaptador intermedio entre `image` y el Framebuffer nativo.
│   └── ui.rs           # Dibujo pixel por pixel de texto, menús, vida y mensajes.
├── maze.txt        # Laberinto Nivel 1
├── maze2.txt       # Laberinto Nivel 2
├── maze3.txt       # Laberinto Nivel 3
└── Cargo.toml      # Configuración de Rust y dependencias
```

## Derechos de Autor y Aclaratoria Legal

**ESTE ES UN PROYECTO ÚNICAMENTE CON PROPÓSITOS EDUCATIVOS Y SIN FINES DE LUCRO.**

### Reconocimientos Musicales:
Toda la música de fondo que conforma la ambientación sonora de este proyecto ("SAT Runner") pertenece y es propiedad intelectual exclusiva de **Taylor Swift**, sus disqueras, compositores asociados y publicadoras musicales correspondientes.

Las pistas utilizadas de forma estrictamente educativa incluyen:
- **Nivel 1:** *Out Of The Woods*
- **Nivel 2:** *Ready For It?*
- **Nivel 3:** *I Did Something Bad*
- **Pantalla de Victoria:** *Long Live*

Ninguno de estos audios se utiliza con intención monetaria ni comercial de ningún tipo. Todos los derechos reservados a **Taylor Swift** y su equipo.

### Hugo Méndez - 241265
