# Front 303 — manual

Front 303 es un instrumento VST3 (y CLAP) con cuatro TB-303, cada una con su propio
secuenciador de 16 pasos, su generador de patrones y su propia métrica, pasando por el
mezclador y los efectos de ReBirth RB-338, ampliados con otros modos (Setekh, Valentine, cinta
IV-X, intervalos) y una reverb de cuatro motores. Las secuencias se pueden transponer desde un
teclado MIDI, y cada 303 y los efectos pueden salir por salidas separadas.

## Instalación (Linux)

```sh
tar xzf Front303-0.3.0-linux-x86_64.tar.gz
cd Front303-0.3.0-linux-x86_64
./install.sh
```

Copia `Front303.vst3` en `~/.vst3` y `Front303.clap` en `~/.clap`. En REAPER: *Options →
Preferences → Plug-ins → VST → Re-scan*. Para desinstalar: `./install.sh --uninstall`.

Necesita Linux x86_64 con glibc 2.30 o posterior (Ubuntu 20.04, Debian 11, Fedora 32 o más
recientes), X11 o XWayland y OpenGL para el panel.

## Cómo suena y cuándo

- **Con el proyecto en marcha**, Front 303 sigue al host: tempo, posición y bucles. Los patrones
  quedan alineados a los compases del proyecto (el paso 1 cae en cada compás). El LED **HOST**
  se enciende.
- **Con el proyecto parado**, el botón **RUN** lo hace sonar por su cuenta. El tempo es el del
  proyecto si el host lo comunica; si no, el del knob **BPM**.
- Los patrones y todos los ajustes se guardan con el proyecto.
- Al cargarlo, solo suena la 303-1; las otras tres empiezan con el canal apagado (botón **ON**
  del mezclador).

## Cada 303

| Control | Qué hace |
|---|---|
| Selector de modelo | **REBIRTH** (el sonido 2.0 de ReBirth), **VINTAGE** (el motor "Vintage" de ReBirth), **DW-303** (modelo del circuito real) u **OPEN303** (el Open303 de Robin Schmidt, en la versión de maddanio) |
| SAW / SQUARE | Forma de onda |
| TUNE | ±24 semitonos, en pasos de semitono |
| FINE | ±100 cents |
| CUT OFF FREQ, RESONANCE, ENV MOD, DECAY, ACCENT | Como en la TB-303 |

### Secuenciador

- **Nota**: clic para alternar entre nota y silencio; arrastrar arriba/abajo o girar la rueda
  para cambiar la nota. Con una escala elegida, la nota se mueve por los grados de la escala.
- **AC** acento, **SL** slide (liga ese paso con el siguiente).
- **LENGTH −/+**: longitud del patrón, de 1 a 16 pasos. Las cuatro 303 pueden tener longitudes
  distintas (polimetría).
- **← →** desplazan el patrón un paso; **CLR** lo borra; **−1 / +1** transponen un semitono;
  **OCT** sube una octava (las notas que se salen del rango se pliegan).
- **PATTERN 1–8**: ocho patrones por voz. Con el transporte en marcha, el cambio espera al
  siguiente compás; el punto rojo marca el patrón que suena.
- **Métrica** (el display bajo LENGTH): cuánto dura cada paso de esa 303 — 1/32, 1/16T,
  1/16, 1/8T, 1/16., 1/8, 1/4T, 1/8. o 1/4. Todas comparten el tempo; con métricas distintas
  (y longitudes distintas) se cruzan polirritmos: por ejemplo una línea en 1/16 contra otra en
  1/8T (seis pasos por compás) o en 1/8. (cada paso dura tres semicorcheas). Las métricas se
  calculan desde el inicio de la canción, así que con el host quedan siempre en el mismo sitio
  del compás.
- **MIDI**: si está encendido, esa 303 sigue la transposición MIDI (ver más abajo).

### Generador

- **GENERATE** escribe un patrón nuevo en el patrón seleccionado, en la escala y la nota raíz
  del panel. Los knobs deciden cómo:
  - **DENSITY**: cuántos pasos suenan.
  - **ACCENTS**: frecuencia de acentos (más probables a contratiempo).
  - **SLIDES**: frecuencia de slides.
  - **OCTAVES**: frecuencia de saltos de octava.
- El generador escribe primero un motivo de medio patrón y lo repite con variaciones, que es lo
  que da a la línea su aire de riff.
- **MUTATE** reescribe una cuarta parte de los pasos.
- **EVOLVE** muta el patrón que suena cada vez que da la vuelta, en la proporción del knob. A
  cero, el patrón no cambia solo.

### Transposición desde MIDI

Front 303 acepta notas MIDI en su entrada. Cada nota que llega transpone las secuencias de las
303 que tengan **MIDI** encendido: **C3 (nota 60)** las deja como están, C#3 sube un semitono,
B2 baja uno, y así hasta ±24 semitonos. La transposición se queda puesta hasta la siguiente
nota (no hace falta mantener la tecla) y cuenta desde el momento exacto en que llega la nota.
El display **MIDI TR** de la cabecera muestra la transposición actual. Se suma después de
cuantizar a la escala: es una transposición literal, como la de una TB-303.

En REAPER basta con armar la pista y elegir la entrada MIDI, o escribir las notas en un ítem
MIDI de la propia pista.

### Escala

**ROOT** y **SCALE** (en la cabecera) cuantizan lo que suena en las cuatro voces y guían al
generador. Hay 14 escalas: cromática (sin cuantizar), mayor, menor, los modos, menor armónica,
frigia dominante, pentatónicas, blues y tonos enteros. Al cuantizar, una nota fuera de la escala
suena como la más cercana de la escala (hacia abajo en caso de empate); el patrón guardado no
cambia.

## Mezclador y efectos

El ruteo es el de ReBirth RB-338, con una reverb añadida:

- **Mezclador**: nivel, panorama, envío al delay (**DLY**), envío a la reverb (**REV**), **ON**
  y **DST** por canal, más **MASTER** en la cabecera.
- **Distortion**: actúa en los canales que tengan **DST** encendido. Dos modos:
  - **REBIRTH**: la de ReBirth, una sola unidad compartida. **SHAPE** a 0 es un recorte duro;
    por encima, curvas de saturación.
  - **SETEKH**: la distorsión Setekh (Full FX Media): realce de agudos de +6 dB a 7,5 kHz,
    mezcla de la señal limpia con tanh según **DRIVE**, y el realce deshecho después. **INPUT**
    da de 0 a +24 dB de entrada, **OUTPUT** de −24 a +24 dB de salida. Cada canal tiene la suya.
- **PCF** (filtro controlado por patrón): se coloca en un canal; 55 patrones rítmicos mueven el
  filtro al tempo. Paso bajo o paso banda.
- **Compressor**: en un canal o en el máster. Dos modos:
  - **REBIRTH**: el de ReBirth (AMOUNT, THRESHOLD).
  - **VALENTINE**: el Valentine de Tote Bag Labs: **COMPRESS** (ganancia de entrada, −6 a
    +36 dB), **RATIO** (1–21; el knee y el umbral siguen al ratio, como en el original),
    **ATTACK** (0,02–55 ms), **RELEASE** (2–1100 ms), **SATURATE** (0 apagado, 1–10),
    **CRUSH** (0 apagado, 1–10: reducción de muestreo y de bits), **MIX** (0–100 %) y
    **OUTPUT** (−24 a +12 dB). Compresor, saturación y recorte suave trabajan a 4× la
    frecuencia. En el máster es estéreo enlazado.
- **Delay**: tiempo de 1 a 32 semicorcheas (o corcheas de tresillo con **TRIPLET**),
  sincronizado al tempo. Tres modos:
  - **REBIRTH**: el de ReBirth, con realimentación y panorama.
  - **TAPE**: el eco de cinta IV-X Delay (Joaquim Broquetas): las curvas del cabezal y la
    respuesta de cada velocidad de cinta y bias son las tablas medidas de IV-X. **TAPE** elige
    15, 7,5 o 3,75 IPS con bias 2–5; **WOW**, **FLUTTER** y **NOISE** (ruido y rascado de
    cinta). Como en IV-X, al cambiar el tiempo la cinta tarda en llegar a la nueva velocidad y
    el eco se desafina por el camino; al activar el modo la cinta "arranca" en algo menos de
    un segundo. **FEEDBACK** llega a la autooscilación (la curva de la cinta la limita).
  - **INTERVALS**: repeticiones sobre un patrón de intervalos, a la manera de Delay Intervals:
    cada repetición se oscurece y baja un poco más que la anterior. **PATTERN** elige el
    patrón (STRAIGHT, PING PONG, DOTTED, TRESILLO, SWELL, GALLOP, SCATTER, STUTTER) con sus
    amplitudes izquierda/derecha; el tiempo es la distancia entre intervalos (hasta 1 s);
    **FALLOFF** cuánto baja cada repetición, **TONE** la oscurece (izquierda) o la adelgaza
    (derecha), **SPREAD** lleva de mono a las amplitudes estéreo del patrón y **LOOP** realimenta
    el final de la línea a su entrada.
- **Reverb**: recibe los envíos **REV** de los canales (después del nivel y del panorama) y,
  con **DLY → REV**, parte del retorno del delay. Cuatro motores, ocho programas cada uno:
  - **HALL**: Dragonfly Hall Reverb (reflexiones tempranas + cola zrev2 de freeverb3), con
    programas de sus bancos (Bright Room … Great Hall).
  - **EARLY**: Dragonfly Early Reflections, con sus ocho programas.
  - **AETHER**: Aether (Dougal Stewart). El original no trae presets: los ocho programas
    (Velvet Hall, Wood Room, Stone Chamber, Cathedral, Bright Plate, Slow Bloom, Dark Cave,
    Endless) son de Front 303.
  - **CLOUD**: CloudReverb (motor CloudSeed), con sus presets (menos Chorus Delay, que no es una
    reverb).

  **PREDELAY** (0–250 ms), **WIDTH** (de mono a 150 %) y **LEVEL** son comunes. **SIZE**,
  **DECAY**, **DAMPING** y **MOD** modifican el programa: en el centro el programa queda tal
  cual; a la izquierda, más pequeño, corto, brillante o quieto; a la derecha, más grande,
  largo, oscuro o modulado. En EARLY no hay cola: DECAY y MOD no actúan. Los programas están
  nivelados entre sí aproximadamente.

## Salidas separadas

El plugin tiene la salida principal estéreo y cinco salidas estéreo auxiliares: una por 303
(canales 3–4, 5–6, 7–8 y 9–10) y una con los retornos de los efectos, delay y reverb (canales
11–12). El selector **OUTPUT** de la cabecera elige:

- **STEREO**: todo sale mezclado por la salida principal (las auxiliares quedan en silencio).
- **MULTI-OUT**: cada 303 sale por su par después de su tira de canal (distorsión, PCF,
  compresor si está en ese canal, nivel y panorama); los retornos de delay y reverb, por el
  suyo. La salida principal queda en silencio y la sección máster (MASTER y compresor en el
  máster) no actúa. Los envíos a los efectos siguen funcionando igual.

En REAPER: en el botón de rutado de la pista de Front 303, desactiva el envío al máster si no
quieres oír la principal, y crea una pista por cada par con *Receives → Front 303 →
audio 3/4 → 1/2*, etc. (o, más rápido, en el menú del plugin *Build multichannel routing for
output of selected FX*). La pista de Front 303 necesita 12 canales (*Track channels: 12*).
Así cada 303 y los efectos se pueden grabar o procesar en su propia pista.

## Uso del panel

- **Knobs**: arrastrar arriba/abajo; con Mayúsculas o Alt, ajuste fino; rueda del ratón;
  doble clic para el valor por defecto. Al pasar por encima se muestra el valor.
- **Displays con flechas** (modelo, métrica, escala, modos de efectos, programas, tiempo del
  delay, salida): clic en el tercio
  izquierdo para retroceder y en el resto para avanzar; rueda del ratón; clic derecho para el
  valor por defecto.
- La ventana se puede redimensionar (mantiene la proporción) si el host lo permite.

Todos los controles, salvo el contenido de los patrones, son parámetros automatizables con
nombres del tipo "303-2 Cutoff" o "PCF Frequency".

## A tener en cuenta

- **CPU del modelo DW-303**: simula el circuito con sobremuestreo y cuesta bastante más que
  los modelos ReBirth y Open303. En la máquina de pruebas (un Xeon a 2,1 GHz) cada voz DW-303 ocupaba en
  torno al 20 % de un núcleo; las cuatro a la vez, casi un núcleo entero. El selector **DW-303
  SOLVER → ECONOMY** apenas lo reduce. Las voces REBIRTH y VINTAGE no cuestan casi nada.
- **Los knobs no se corresponden exactamente entre modelos**: cada uno conserva sus curvas
  (Open303 usa las de su propio plugin: cutoff 314–2394 Hz, decay 200–2000 ms). Afinación y
  volumen sí están igualados.
- **Open303** tiene menos pico de resonancia y acentos más planos que una TB-303 real (en
  las medidas: 6–21 dB de pico frente a 19–43 dB, y los acentos no se acumulan); se incluye tal
  cual, con dos accesos fuera de rango del original corregidos.
- **El DW-303 se apaga solo** con notas muy largas, como el circuito; los modelos ReBirth
  sostienen mientras la puerta está abierta.
- **La distorsión deja algo de continua** con diente de sierra, como en ReBirth.
- **El motor trabaja siempre a 44,1 kHz** y se remuestrea a la frecuencia del proyecto, sin
  latencia añadida. Las reverbs y los demás efectos también corren a 44,1 kHz (el eco de cinta
  a 176,4 kHz: IV-X trabaja a 192 kHz, y su filtro de cabezal queda un 8 % más grave).
- **El ruido de cinta de IV-X es sintético**: las grabaciones de ruido del original no están
  en su código fuente publicado; Front 303 genera un ruido blanco oscurecido según la
  velocidad de cinta.
- **En MULTI-OUT las salidas de las 303 no pasan por MASTER**: pueden superar 0 dBFS si los
  niveles de canal están altos (en coma flotante no hay recorte; baja LEVEL o la pista).
- **Solo cuenta el note-on**: el note-off no deshace la transposición.
