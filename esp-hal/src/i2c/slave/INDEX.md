# I2C Slave Driver - Quick Navigation

**Complete I2C Slave Driver Implementation for ESP32 Chips**

---

## 📚 Start Here

### New Users
👉 **[QUICKSTART.md](QUICKSTART.md)** - Get started in 5 minutes

### Understanding the Driver
👉 **[README.md](README.md)** - Complete overview and user guide

### Code Examples
👉 **[EXAMPLE.md](EXAMPLE.md)** - Working code examples

---

## 🔧 For Developers

### Architecture & Design
👉 **[DESIGN.md](DESIGN.md)** - Design decisions and internals

### Implementation
👉 **[mod.rs](mod.rs)** - Main driver code (~1800 lines)

### Testing
👉 **[TESTING.md](TESTING.md)** - Test checklist (45+ scenarios)
👉 **[test-suite/TEST_SUITE_SUMMARY.md](test-suite/TEST_SUITE_SUMMARY.md)** - Complete test suite (207+ tests)

---

## 📂 File Organization

```
slave/
├── INDEX.md              ← YOU ARE HERE
├── mod.rs                ← Driver implementation
│
├── README.md             ← Start for general users
├── QUICKSTART.md         ← Fastest way to get started
├── EXAMPLE.md            ← Working code examples
├── DESIGN.md             ← For developers/contributors
├── TESTING.md            ← Test checklist
├── FILE_SUMMARY.md       ← Complete file inventory
│
└── test-suite/           ← 207+ tests
    ├── README.md         ← Test suite guide
    ├── TEST_SUITE_SUMMARY.md  ← Test metrics
    ├── unit/             ← 53+ unit tests
    ├── functional/       ← 77+ functional tests
    ├── async_tests/      ← 23+ async tests
    ├── performance/      ← 15+ performance tests
    ├── reliability/      ← 18+ reliability tests
    ├── integration/      ← 21+ integration tests
    └── helpers/          ← Test utilities
```

---

## 🎯 Quick Links by Purpose

### I Want To...

**...Get Started Quickly**
- [QUICKSTART.md](QUICKSTART.md) - 5-minute start
- [EXAMPLE.md](EXAMPLE.md) - Copy-paste examples

**...Understand How It Works**
- [README.md](README.md) - Feature overview
- [DESIGN.md](DESIGN.md) - Architecture details

**...Configure the Driver**
- [README.md#Configuration](README.md) - Configuration options
- [EXAMPLE.md](EXAMPLE.md) - Configuration examples

**...Use Async Mode**
- [README.md#Async](README.md) - Async overview
- [EXAMPLE.md](EXAMPLE.md) - Async examples
- [test-suite/async_tests/](test-suite/async_tests/) - Async test examples

**...Handle Interrupts**
- [README.md#Interrupts](README.md) - Interrupt guide
- [EXAMPLE.md](EXAMPLE.md) - Interrupt examples

**...Test the Driver**
- [TESTING.md](TESTING.md) - Manual test checklist
- [test-suite/README.md](test-suite/README.md) - Automated tests
- [test-suite/TEST_SUITE_SUMMARY.md](test-suite/TEST_SUITE_SUMMARY.md) - Test details

**...Contribute or Modify**
- [DESIGN.md](DESIGN.md) - Design rationale
- [FILE_SUMMARY.md](FILE_SUMMARY.md) - File inventory
- [mod.rs](mod.rs) - Implementation code

**...Troubleshoot Issues**
- [DESIGN.md#Troubleshooting](DESIGN.md) - Common issues
- [README.md#Limitations](README.md) - Known limitations

**...See Performance**
- [test-suite/performance/](test-suite/performance/) - Performance tests
- [DESIGN.md#Performance](DESIGN.md) - Performance notes

---

## 📊 Statistics

| Metric | Count |
|--------|-------|
| **Total Files** | 33 |
| **Lines of Code** | ~9,700+ |
| **Driver Code** | ~1,800 lines |
| **Documentation** | ~2,900 lines |
| **Test Code** | ~5,000+ lines |
| **Total Tests** | 207+ |
| **HIL Tests** | 105+ |
| **Unit Tests** | 102+ |
| **Supported Chips** | 6 (ESP32, S2, S3, C3, C6, H2) |

---

## ✨ Features at a Glance

✅ **Blocking Mode** - Simple, synchronous operations  
✅ **Async Mode** - Non-blocking with Embassy integration  
✅ **7-bit Addressing** - Standard I2C addressing  
✅ **Clock Stretching** - Configurable per chip  
✅ **Signal Filtering** - Noise immunity  
✅ **Interrupt Support** - Efficient event handling  
✅ **FIFO Management** - 32-byte hardware buffer  
✅ **Error Recovery** - Comprehensive error handling  
✅ **Runtime Config** - Change config on the fly  
✅ **All ESP32 Variants** - ESP32, S2, S3, C3, C6, H2  

---

## 🚀 Typical Usage Flow

1. **Read** [QUICKSTART.md](QUICKSTART.md) (5 minutes)
2. **Review** code in [EXAMPLE.md](EXAMPLE.md) (10 minutes)
3. **Copy** example to your project
4. **Configure** for your hardware
5. **Test** using [TESTING.md](TESTING.md) checklist
6. **Deploy** with confidence!

---

## 🆘 Getting Help

1. **Check documentation** in this directory
2. **Review examples** in [EXAMPLE.md](EXAMPLE.md)
3. **Read troubleshooting** in [DESIGN.md](DESIGN.md)
4. **Search test cases** in [test-suite/](test-suite/)
5. **Check repository issues** (if open source)

---

## 📝 Document Purposes

| Document | Best For | Reading Time |
|----------|----------|--------------|
| **INDEX.md** | Navigation | 2 min |
| **QUICKSTART.md** | Getting started fast | 5 min |
| **README.md** | Complete overview | 15 min |
| **EXAMPLE.md** | Copy-paste code | 10 min |
| **DESIGN.md** | Deep understanding | 30 min |
| **TESTING.md** | QA/Testing | 20 min |
| **FILE_SUMMARY.md** | File inventory | 10 min |
| **TEST_SUITE_SUMMARY.md** | Test details | 15 min |

---

## 🔄 Last Updated

**Date:** October 9, 2025  
**Version:** 1.0.0 (Initial Complete Implementation)  
**Status:** ✅ Production Ready (pending hardware validation)

---

## 📄 License

This driver is part of esp-hal and is dual-licensed:
- MIT License
- Apache License 2.0

---

**Happy coding! 🎉**

For questions or contributions, see the main esp-hal repository.
