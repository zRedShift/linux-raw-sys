#include "bluetooth-base.h"

#ifndef ETH_ALEN
#define ETH_ALEN 6
#endif

#define HCI_DATA_DIR 1
#define HCI_FILTER 2
#define HCI_TIME_STAMP 3

#define HCI_CMSG_DIR 0x01
#define HCI_CMSG_TSTAMP 0x02

struct sockaddr_hci {
    sa_family_t hci_family;
    unsigned short hci_dev;
    unsigned short hci_channel;
};

#define HCI_DEV_NONE 0xffff
#define HCI_CHANNEL_RAW 0
#define HCI_CHANNEL_USER 1
#define HCI_CHANNEL_MONITOR 2
#define HCI_CHANNEL_CONTROL 3
#define HCI_CHANNEL_LOGGING 4

struct hci_filter {
    unsigned long type_mask;
    unsigned long event_mask[2];
    __le16 opcode;
};

struct hci_ufilter {
    __u32 type_mask;
    __u32 event_mask[2];
    __le16 opcode;
};

#define HCI_FLT_TYPE_BITS 31
#define HCI_FLT_EVENT_BITS 63
#define HCI_FLT_OGF_BITS 63
#define HCI_FLT_OCF_BITS 127

#define HCIDEVUP _IOW('H', 201, int)
#define HCIDEVDOWN _IOW('H', 202, int)
#define HCIDEVRESET _IOW('H', 203, int)
#define HCIDEVRESTAT _IOW('H', 204, int)
#define HCIGETDEVLIST _IOR('H', 210, int)
#define HCIGETDEVINFO _IOR('H', 211, int)
#define HCIGETCONNLIST _IOR('H', 212, int)
#define HCIGETCONNINFO _IOR('H', 213, int)
#define HCIGETAUTHINFO _IOR('H', 215, int)
#define HCISETRAW _IOW('H', 220, int)
#define HCISETSCAN _IOW('H', 221, int)
#define HCISETAUTH _IOW('H', 222, int)
#define HCISETENCRYPT _IOW('H', 223, int)
#define HCISETPTYPE _IOW('H', 224, int)
#define HCISETLINKPOL _IOW('H', 225, int)
#define HCISETLINKMODE _IOW('H', 226, int)
#define HCISETACLMTU _IOW('H', 227, int)
#define HCISETSCOMTU _IOW('H', 228, int)
#define HCIBLOCKADDR _IOW('H', 230, int)
#define HCIUNBLOCKADDR _IOW('H', 231, int)
#define HCIINQUIRY _IOR('H', 240, int)

struct hci_dev_stats {
    __u32 err_rx;
    __u32 err_tx;
    __u32 cmd_tx;
    __u32 evt_rx;
    __u32 acl_tx;
    __u32 acl_rx;
    __u32 sco_tx;
    __u32 sco_rx;
    __u32 byte_rx;
    __u32 byte_tx;
};

struct hci_dev_info {
    __u16 dev_id;
    char name[8];
    bdaddr_t bdaddr;
    __u32 flags;
    __u8 type;
    __u8 features[8];
    __u32 pkt_type;
    __u32 link_policy;
    __u32 link_mode;
    __u16 acl_mtu;
    __u16 acl_pkts;
    __u16 sco_mtu;
    __u16 sco_pkts;
    struct hci_dev_stats stat;
};

struct hci_conn_info {
    __u16 handle;
    bdaddr_t bdaddr;
    __u8 type;
    __u8 out;
    __u16 state;
    __u32 link_mode;
};

struct hci_dev_req {
    __u16 dev_id;
    __u32 dev_opt;
};

struct hci_dev_list_req {
    __u16 dev_num;
    struct hci_dev_req dev_req[] __counted_by(dev_num);
};

struct hci_conn_list_req {
    __u16 dev_id;
    __u16 conn_num;
    struct hci_conn_info conn_info[];
};

struct hci_conn_info_req {
    bdaddr_t bdaddr;
    __u8 type;
    struct hci_conn_info conn_info[];
};

struct hci_auth_info_req {
    bdaddr_t bdaddr;
    __u8 type;
};

struct hci_inquiry_req {
    __u16 dev_id;
    __u16 flags;
    __u8 lap[3];
    __u8 length;
    __u8 num_rsp;
};

#define IREQ_CACHE_FLUSH 0x0001

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
