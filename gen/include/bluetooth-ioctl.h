#include "bluetooth-base.h"

#include "../linux/include/net/bluetooth/hci_sock.h"

#ifndef ETH_ALEN
#define ETH_ALEN 6
#endif

#define RFCOMMCREATEDEV _IOW('R', 200, int)
#define RFCOMMRELEASEDEV _IOW('R', 201, int)
#define RFCOMMGETDEVLIST _IOR('R', 210, int)
#define RFCOMMGETDEVINFO _IOR('R', 211, int)
#define RFCOMMSTEALDLC _IOW('R', 220, int)

#define RFCOMM_REUSE_DLC 0
#define RFCOMM_RELEASE_ONHUP 1
#define RFCOMM_HANGUP_NOW 2
#define RFCOMM_TTY_ATTACHED 3
#define RFCOMM_DEFUNCT_BIT4 4
#define RFCOMM_DEV_RELEASED 0
#define RFCOMM_TTY_OWNED 1

struct rfcomm_dev_req {
    s16 dev_id;
    u32 flags;
    bdaddr_t src;
    bdaddr_t dst;
    u8 channel;
};

struct rfcomm_dev_info {
    s16 id;
    u32 flags;
    u16 state;
    bdaddr_t src;
    bdaddr_t dst;
    u8 channel;
};

struct rfcomm_dev_list_req {
    u16 dev_num;
    struct rfcomm_dev_info dev_info[] __counted_by(dev_num);
};

#define BNEPCONNADD _IOW('B', 200, int)
#define BNEPCONNDEL _IOW('B', 201, int)
#define BNEPGETCONNLIST _IOR('B', 210, int)
#define BNEPGETCONNINFO _IOR('B', 211, int)
#define BNEPGETSUPPFEAT _IOR('B', 212, int)

#define BNEP_SETUP_RESPONSE 0
#define BNEP_SETUP_RSP_SENT 10

struct bnep_connadd_req {
    int sock;
    __u32 flags;
    __u16 role;
    char device[16];
};

struct bnep_conndel_req {
    __u32 flags;
    __u8 dst[ETH_ALEN];
};

struct bnep_conninfo {
    __u32 flags;
    __u16 role;
    __u16 state;
    __u8 dst[ETH_ALEN];
    char device[16];
};

struct bnep_connlist_req {
    __u32 cnum;
    struct bnep_conninfo __user *ci;
};

#define CMTPCONNADD _IOW('C', 200, int)
#define CMTPCONNDEL _IOW('C', 201, int)
#define CMTPGETCONNLIST _IOR('C', 210, int)
#define CMTPGETCONNINFO _IOR('C', 211, int)

#define CMTP_LOOPBACK 0

struct cmtp_connadd_req {
    int sock;
    __u32 flags;
};

struct cmtp_conndel_req {
    bdaddr_t bdaddr;
    __u32 flags;
};

struct cmtp_conninfo {
    bdaddr_t bdaddr;
    __u32 flags;
    __u16 state;
    int num;
};

struct cmtp_connlist_req {
    __u32 cnum;
    struct cmtp_conninfo __user *ci;
};

#define HIDPCONNADD _IOW('H', 200, int)
#define HIDPCONNDEL _IOW('H', 201, int)
#define HIDPGETCONNLIST _IOR('H', 210, int)
#define HIDPGETCONNINFO _IOR('H', 211, int)

#define HIDP_VIRTUAL_CABLE_UNPLUG 0
#define HIDP_BOOT_PROTOCOL_MODE 1
#define HIDP_BLUETOOTH_VENDOR_ID 9
#define HIDP_WAITING_FOR_RETURN 10
#define HIDP_WAITING_FOR_SEND_ACK 11

struct hidp_connadd_req {
    int ctrl_sock;
    int intr_sock;
    __u16 parser;
    __u16 rd_size;
    __u8 __user *rd_data;
    __u8 country;
    __u8 subclass;
    __u16 vendor;
    __u16 product;
    __u16 version;
    __u32 flags;
    __u32 idle_to;
    char name[128];
};

struct hidp_conndel_req {
    bdaddr_t bdaddr;
    __u32 flags;
};

struct hidp_conninfo {
    bdaddr_t bdaddr;
    __u32 flags;
    __u16 state;
    __u16 vendor;
    __u16 product;
    __u16 version;
    char name[128];
};

struct hidp_connlist_req {
    __u32 cnum;
    struct hidp_conninfo __user *ci;
};
