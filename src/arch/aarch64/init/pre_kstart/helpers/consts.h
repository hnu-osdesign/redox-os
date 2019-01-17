#define PAGE_SIZE               4096
#define BLOCK_SIZE              0x40000000
#define VIRT_BITS               48

#define DTB_VBASE               0xfffffd0000000000                  // By convention DTBs are mapped here. Must tally with consts.rs.
#define DTB_MAX_SIZE            0x200000

#define ENV_VBASE               0xfffffc0000000000                  // Where the boot environment is setup for the kernel
#define ENV_MAX_SIZE            (PAGE_SIZE)

#define EARLY_KSTACK_SIZE       (PAGE_SIZE)                         // Initial stack

#define SCTLR_M                 0x00000001                          // SCTLR_M bit used to control MMU on/off

#define DEVICE_MEM              0                                   // Memory type specifiers
#define NORMAL_UNCACHED_MEM     1
#define NORMAL_CACHED_MEM       2

#define DESC_VALID_BIT          0                                   // Descriptor validity setting
#define DESC_VALID              1
#define DESC_INVALID            0

#define DESC_TYPE_BIT           1                                   // Descriptor type
#define DESC_TYPE_TABLE         1
#define DESC_TYPE_PAGE          1
#define DESC_TYPE_BLOCK         0

#define BLOCK_DESC_MASK         (~((0xffff << 48) | (0xffff)))      // Convenience mask for block desciptors
#define ACCESS_FLAG_BIT         (1 << 10)

// To get access to an early console over a PL011 or compatible UART,
// define DEBUG_UART and modify the values below to suit your platform.

#ifdef DEBUG_UART
#define DEVMAP_PBASE            0x00000000                          // These are platform specific ranges where interesting
#define DEVMAP_SIZE             0x40000000                          // peripherals lie. Change to suit the platform of interest.
                                                                    // Only needed to map in a diagnostic UART.

#define UART_VBASE              0xfffffe0009000000                  // Change this to get an early console for debugging
#define UART_PBASE              0x09000000                          // Only a PL011 or compatible UART is supported
#define UART_SIZE               0x2000                              // The code will likely need mods
                                                                    // A properly configured console is setup later
                                                                    // The UART_VBASE should be a suitable offset into the DEVMAP VA region
                                                                    // as defined in consts.rs The values here are for qemu-system-aarch64-virt
#define NUM_L2_TABLES           13                                  // There are normally 12 tables to clear (2 L0, 5 L1, 5 L2, 1 env)
#else                                                               // They become 13 if the debug UART is used.
#define NUM_L2_TABLES           12
#endif
