# Allianz-Desk: WhatsApp USB-C Bridge (Rust) 📱⚡

Controlador y lector bidireccional de WhatsApp para Android a través de conexión física **USB-C** (ADB), desarrollado 100% en **Rust**.

Permite leer notificaciones y mensajes entrantes de WhatsApp en tiempo real, visualizar conversaciones en una interfaz de consola interactiva y responder mensajes o iniciar chats directos desde la PC sin necesidad de emuladores pesados ni dependencias externas.

---

## 🚀 Características Principales

- **Lectura en Tiempo Real:** Inspección y parseo estructurado de notificaciones del subsistema de Android (`dumpsys notification`) para capturar remitentes, grupos y contenido de mensajes de WhatsApp.
- **Respuesta y Envío Directo:**
  - Envío automático mediante Intents nativos de Android (`https://api.whatsapp.com/send?phone=...&text=...`).
  - Escritura y despacho directo en chats activos en pantalla vía inyección de eventos ADB.
- **Cero Dependencias Externas Requeridas:** Incorpora detección automática de `adb.exe` local (`platform-tools/`) y soporte para rutas personalizadas.
- **Rendimiento Nativo:** Construido en Rust para un consumo mínimo de recursos de CPU/RAM y latencia casi nula.
- **Interfaz Interactiva (CLI / TUI):** Menú en terminal con códigos de color, visualización limpia de mensajes recibidos y opciones de respuesta rápida.

---

## 🛠️ Arquitectura del Proyecto

```text
whatsapp_usb_bridge/
├── Cargo.toml                  # Dependencias y configuración del paquete Rust
├── platform-tools/             # Binarios portables de Android Debug Bridge (adb.exe)
└── src/
    ├── main.rs                 # Loop interactivo, gestión de eventos y CLI
    ├── adb/
    │   ├── mod.rs              # Exportación del cliente ADB
    │   └── device.rs           # Gestión de conexión USB, shell y envío de teclas/taps
    ├── whatsapp/
    │   ├── mod.rs              # Modelos y motores de WhatsApp
    │   ├── models.rs           # Estructura de datos WhatsAppMessage
    │   ├── parser.rs           # Parser regex para extracción de notificaciones
    │   └── sender.rs           # Despacho de mensajes (URL encode, Intents y simulación de teclado)
    └── ui/
        ├── mod.rs              # Módulo de interfaz de usuario
        └── terminal.rs         # Renderizado de banners, bandeja y menús con estilos de color
```

---

## 📋 Requisitos Previos

### 1. En el Celular Android
1. Conectar el celular a la PC mediante cable **USB-C**.
2. Ir a **Ajustes > Acerca del teléfono** y pulsar 7 veces sobre **Número de compilación** para habilitar las **Opciones de desarrollador**.
3. Ir a **Ajustes > Opciones de desarrollador** y activar **Depuración por USB**.
4. Al conectar por primera vez, marcar la casilla **"Permitir siempre desde esta computadora"** y aceptar la clave RSA.

### 2. En la PC
- **Rust Toolchain:** Cargo 1.80+ (Rust 2021 edition).
- Windows 10/11, Linux o macOS.

---

## ⚡ Instalación y Uso

### Compilación y Ejecución en Modo Release

```powershell
# Clonar o situarse en el directorio del proyecto
cd whatsapp_usb_bridge

# Ejecutar en modo release optimizado
cargo run --release
```

### Opciones de Línea de Comandos (CLI)

```text
Uso: whatsapp_usb_bridge.exe [OPCIONES]

Opciones:
  -a, --adb-path <ADB_PATH>    Ruta personalizada al ejecutable adb.exe (opcional)
  -d, --device <DEVICE>        Serial del dispositivo específico si hay múltiples conectados
  -p, --poll-secs <POLL_SECS>  Intervalo de sondeo en segundos [por defecto: 3]
  -h, --help                   Muestra información de ayuda
  -V, --version                Muestra la versión
```

Ejemplo con ruta específica de ADB:
```powershell
cargo run --release -- --adb-path "C:\ruta\hacia\adb.exe"
```

---

## 🎮 Menú Interactivo

Una vez iniciado el programa, verás el estado del dispositivo conectado y las opciones:

```text
==========================================================
      WHATSAPP USB-C BRIDGE (ANDROID <-> RUST CLI)       
==========================================================
● Conectado por USB: Samsung Galaxy S23 (R5CT...) [Estado: device]
----------------------------------------------------------

--- BANDEJA DE MENSAJES ENTRANTES ---
[1] [Directo] Juan Perez (14:30:15):
    💬 ¿Hola, pudiste revisar el archivo?

----------------------------------------------------------
ACCIONES DISPONIBLES:
  [1] Responder a un mensaje reciente
  [2] Enviar mensaje directo a un número
  [3] Refrescar bandeja de mensajes
  [4] Salir
----------------------------------------------------------
Selecciona una opción (1-4): 
```

- **Opción 1:** Selecciona el índice del mensaje para redactar una respuesta inmediata.
- **Opción 2:** Ingresa un número de teléfono internacional (ej. `+5491122334455`) para abrir y enviar el mensaje automáticamente.
- **Opción 3:** Fuerza la actualización de la bandeja consultando notificaciones pendientes.

---

## 🔒 Privacidad y Seguridad

- Todo el procesamiento de datos se realiza **100% de manera local** a través del cable USB-C.
- No se envían credenciales, datos de chats ni tokens a servidores de terceros.
- No requiere permisos de Root en el dispositivo móvil.

---

## 📄 Licencia

Este proyecto se distribuye bajo la licencia MIT. Consulta el archivo `LICENSE` para más detalles.
