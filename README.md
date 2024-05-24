# cpu-meter
cpu-meter is a classical analog gauge that shows a cpu usage. It is a simple analog voltmeter with a microprocessor. Any MCU with USB serial communication and PWM GPIO could be used.

<p align=center><img src='https://github.com/luftaquila/cpu-meter/assets/17094868/03c583aa-1326-47a0-9ed3-b35024430d33' width=70%></p>

Currently, cpu-meter supports Windows, MacOS, and Linux (not tested yet).\
Any MCUs like Arduino(ATmega), STM32 and ESP32 dev board with USB can be used.

## Prerequisites

* An analog voltmeter
* Any MCU with USB serial and PWM output

> [!TIP]
> There are common analog panel meters called `85C1` and `91C4` series, which cost only $2~3 and have a various voltage range. There are 0-3V and 0-5V models, which fit with most common MCUs.

Choose a voltage range by your MCU's logic level. 5V MCUs such as Arduino can manipulate both 0-5V and 0-3V models. 3.3V MCUs such as ESP32 or STM32, Raspberry Pi, etc. can only run 0-3V models.

> [!IMPORTANT]  
> If you are using a 5V MCU with a 0-3V voltmeter, the PWM output ***must*** be regulated to 3V max. If your MCU is 3.3V, it is recommended to adjust the PWM voltage to a max 3V, not 3.3V.

> [!IMPORTANT]  
> Please keep in mind that if you use a 0-5V model, the PWM output voltage could be lower than 5V depending on your PC and MCU.
> <br><br>
> If your PC supplies a voltage less than 5V to the MCU via the USB, the MCU can only output that voltage to its PWM pin.
> Also, most MCUs have a protection diode on their USB input, which makes additional VCC voltage drop. Mostly 0.3V (Schottky diode) or 0.7V (general diode).
> <br><br>
> This will result in the analog gauge not pointing its max position, even if the CPU usage is 100%. For example, if your PC supplies 4.7V and a protection diode drops 0.7V, the gauge will point 4V position even if the CPU is in full load.

<br>

## DIY!
1. Connect voltmeter and MCU. Test your voltmeter's polarity with some batteries before connect. + terminal goes to the PWM supported GPIO pin, and - terminal goes to the GND.
2. Upload a firmware in [`devices/*`](https://github.com/luftaquila/cpu-meter/tree/main/devices) directory to your MCU. Check some parameters (PWM pin, max voltage, etc.) in the code before uploading.
3. Download a desktop executable at [Releases](https://github.com/luftaquila/cpu-meter/releases).

> [!NOTE]
> For now, only firmware for Arduino is implemented. BTW, it is a basic program that reads 4 bytes from 115200 bps serial (Little-Endian float value) and makes corresponding PWM output. Please contribute!

Optional:
* Build a housing with 3d print
* You can build a desktop executable on your own by running `cargo build --release`.

## Run
1. Connect your MCU and PC with a USB cable.
2. Run your desktop executable. The tray icon will be generated.
3. Click the tray icon and select your MCU's serial port.

## More
If you are good at soldering, something like this is possible:

<p align=center><img src='https://github.com/luftaquila/cpu-meter/assets/17094868/64b86bb4-c493-46ee-a92e-635a30d26e69' height=300px> <img src='https://github.com/luftaquila/cpu-meter/assets/17094868/f41067a8-71cc-41a4-b089-065da2732e4b' height=300px> <img src='https://github.com/luftaquila/cpu-meter/assets/17094868/11d534f2-2b8d-4f98-adbc-e1453bd10fb8' height=300px></p>

This is a cpu-meter with a USB Type-C port, all components included inside.

The analog gauge is 0-5V 85C1 voltmeter and the red board is an Arduino Pico. USB Micro port on Pico is desoldered and jumped onto a DIY Type-C board at the back side.
