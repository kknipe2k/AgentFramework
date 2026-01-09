// Counter module
class Counter {
  constructor(initial = 0) {
    this.value = initial;
  }

  increment() {
    this.value++;
    return this.value;
  }

  decrement() {
    this.value--;
    return this.value;
  }

  reset() {
    this.value = 0;
    return this.value;
  }
}

module.exports = Counter;
