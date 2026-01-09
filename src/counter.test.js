const Counter = require('./counter');

describe('Counter', () => {
  test('starts at 0 by default', () => {
    const counter = new Counter();
    expect(counter.value).toBe(0);
  });

  test('starts at initial value', () => {
    const counter = new Counter(10);
    expect(counter.value).toBe(10);
  });

  test('increments', () => {
    const counter = new Counter();
    expect(counter.increment()).toBe(1);
    expect(counter.increment()).toBe(2);
  });

  test('decrements', () => {
    const counter = new Counter(5);
    expect(counter.decrement()).toBe(4);
  });

  test('resets to 0', () => {
    const counter = new Counter(100);
    expect(counter.reset()).toBe(0);
  });
});
