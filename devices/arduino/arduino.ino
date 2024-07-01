// common PWM pins shared by various arduino series
const int pwm[] = { 3, 5, 6, 9, 10, 11 };
const int pwm_cnt = (sizeof(pwm) / sizeof(const int));

union f32 {
  byte bytes[4];
  float value;
};

void setup() {
  Serial.begin(115200);
}

void loop() {
  f32 f;
  
  if (Serial.available() >= 4) {
    for (int i = 0; i < 4; i++) {
      f.bytes[i] = Serial.read();
    }

    /*
     * packet value is a 4 byte little endian float percentage
     * first byte in the packet designates the pwm output port
     *
     * |       < PACKET STRUCTURE >       |
     * |  byte 0  byte 1  byte 2  byte 3  |
     * | | port |                         |
     * | |   value (float percentage)   | |
     *
     * port uses the part of the fraction bits of the IEEE 754 float structure
     * this sacrifies the precision of the value, but it won't visible in the meter
     */

    if (f.bytes[0] < pwm_cnt) {
      // analogWrite max 255 when f.value is 100.0 %
      analogWrite(pwm[f.bytes[0]], int(2.55 * f.value));
    }
  }
}
