#define PWM 3

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
    analogWrite(PWM, int(2.55 * f.value));
  }
}