// Linux Bluetooth userspace ABI surface assembled from kernel Bluetooth
// headers under include/net/bluetooth/ and net/bluetooth/.

#include "support.h"
#include "bluetooth-ioctl.h"

#define BT_SUBSYS_VERSION 2
#define BT_SUBSYS_REVISION 22

#ifndef AF_BLUETOOTH
#define AF_BLUETOOTH 31
#define PF_BLUETOOTH AF_BLUETOOTH
#endif

#define BLUETOOTH_VER_1_1 1
#define BLUETOOTH_VER_1_2 2
#define BLUETOOTH_VER_2_0 3
#define BLUETOOTH_VER_2_1 4
#define BLUETOOTH_VER_4_0 6

#define BTPROTO_L2CAP 0
#define BTPROTO_HCI 1
#define BTPROTO_SCO 2
#define BTPROTO_RFCOMM 3
#define BTPROTO_BNEP 4
#define BTPROTO_CMTP 5
#define BTPROTO_HIDP 6
#define BTPROTO_AVDTP 7
#define BTPROTO_ISO 8
#define BTPROTO_LAST BTPROTO_ISO

#define SOL_HCI 0
#define SOL_L2CAP 6
#define SOL_SCO 17
#define SOL_RFCOMM 18

#define BT_SECURITY 4
struct bt_security {
    __u8 level;
    __u8 key_size;
};
#define BT_SECURITY_SDP 0
#define BT_SECURITY_LOW 1
#define BT_SECURITY_MEDIUM 2
#define BT_SECURITY_HIGH 3
#define BT_SECURITY_FIPS 4

#define BT_DEFER_SETUP 7
#define BT_FLUSHABLE 8
#define BT_FLUSHABLE_OFF 0
#define BT_FLUSHABLE_ON 1

#define BT_POWER 9
struct bt_power {
    __u8 force_active;
};
#define BT_POWER_FORCE_ACTIVE_OFF 0
#define BT_POWER_FORCE_ACTIVE_ON 1

#define BT_CHANNEL_POLICY 10
#define BT_CHANNEL_POLICY_BREDR_ONLY 0
#define BT_CHANNEL_POLICY_BREDR_PREFERRED 1
#define BT_CHANNEL_POLICY_AMP_PREFERRED 2

#define BT_VOICE 11
struct bt_voice {
    __u16 setting;
};
#define BT_VOICE_TRANSPARENT 0x0003
#define BT_VOICE_CVSD_16BIT 0x0060
#define BT_VOICE_TRANSPARENT_16BIT 0x0063

#define BT_SNDMTU 12
#define BT_RCVMTU 13
#define BT_PHY 14
#define BT_PHY_BR_1M_1SLOT BIT(0)
#define BT_PHY_BR_1M_3SLOT BIT(1)
#define BT_PHY_BR_1M_5SLOT BIT(2)
#define BT_PHY_EDR_2M_1SLOT BIT(3)
#define BT_PHY_EDR_2M_3SLOT BIT(4)
#define BT_PHY_EDR_2M_5SLOT BIT(5)
#define BT_PHY_EDR_3M_1SLOT BIT(6)
#define BT_PHY_EDR_3M_3SLOT BIT(7)
#define BT_PHY_EDR_3M_5SLOT BIT(8)
#define BT_PHY_LE_1M_TX BIT(9)
#define BT_PHY_LE_1M_RX BIT(10)
#define BT_PHY_LE_2M_TX BIT(11)
#define BT_PHY_LE_2M_RX BIT(12)
#define BT_PHY_LE_CODED_TX BIT(13)
#define BT_PHY_LE_CODED_RX BIT(14)
#define BT_MODE 15
#define BT_MODE_BASIC 0x00
#define BT_MODE_ERTM 0x01
#define BT_MODE_STREAMING 0x02
#define BT_MODE_LE_FLOWCTL 0x03
#define BT_MODE_EXT_FLOWCTL 0x04

#define BT_PKT_STATUS 16
#define BT_SCM_PKT_STATUS 0x03
#define BT_SCM_ERROR 0x04

#define BT_ISO_QOS 17
#define BT_ISO_QOS_CIG_UNSET 0xff
#define BT_ISO_QOS_CIS_UNSET 0xff
#define BT_ISO_QOS_BIG_UNSET 0xff
#define BT_ISO_QOS_BIS_UNSET 0xff
#define BT_ISO_SYNC_TIMEOUT 0x07d0

struct bt_iso_io_qos {
    __u32 interval;
    __u16 latency;
    __u16 sdu;
    __u8 phy;
    __u8 rtn;
};

struct bt_iso_ucast_qos {
    __u8 cig;
    __u8 cis;
    __u8 sca;
    __u8 packing;
    __u8 framing;
    struct bt_iso_io_qos in;
    struct bt_iso_io_qos out;
};

struct bt_iso_bcast_qos {
    __u8 big;
    __u8 bis;
    __u8 sync_factor;
    __u8 packing;
    __u8 framing;
    struct bt_iso_io_qos in;
    struct bt_iso_io_qos out;
    __u8 encryption;
    __u8 bcode[16];
    __u8 options;
    __u16 skip;
    __u16 sync_timeout;
    __u8 sync_cte_type;
    __u8 mse;
    __u16 timeout;
};

struct bt_iso_qos {
    union {
        struct bt_iso_ucast_qos ucast;
        struct bt_iso_bcast_qos bcast;
    };
};

#define BT_ISO_PHY_1M BIT(0)
#define BT_ISO_PHY_2M BIT(1)
#define BT_ISO_PHY_CODED BIT(2)
#define BT_ISO_PHY_ANY (BT_ISO_PHY_1M | BT_ISO_PHY_2M | BT_ISO_PHY_CODED)

#define BT_CODEC 19
struct bt_codec_caps {
    __u8 len;
    __u8 data[];
} __packed;

struct bt_codec {
    __u8 id;
    __u16 cid;
    __u16 vid;
    __u8 data_path;
    __u8 num_caps;
} __packed;

struct bt_codecs {
    __u8 num_codecs;
    struct bt_codec codecs[];
} __packed;

#define BT_CODEC_CVSD 0x02
#define BT_CODEC_TRANSPARENT 0x03
#define BT_CODEC_MSBC 0x05
#define BT_ISO_BASE 20
#define BT_PKT_SEQNUM 22
#define BT_SCM_PKT_SEQNUM 0x05

#define L2CAP_DEFAULT_MTU 672
#define L2CAP_DEFAULT_MIN_MTU 48
#define L2CAP_DEFAULT_FLUSH_TO 0xFFFF
#define L2CAP_EFS_DEFAULT_FLUSH_TO 0xFFFFFFFF
#define L2CAP_DEFAULT_TX_WINDOW 63
#define L2CAP_DEFAULT_EXT_WINDOW 0x3FFF
#define L2CAP_DEFAULT_MAX_TX 3
#define L2CAP_DEFAULT_RETRANS_TO 2
#define L2CAP_DEFAULT_MONITOR_TO 12
#define L2CAP_DEFAULT_MAX_PDU_SIZE 1492
#define L2CAP_DEFAULT_ACK_TO 200
#define L2CAP_DEFAULT_MAX_SDU_SIZE 0xFFFF
#define L2CAP_DEFAULT_SDU_ITIME 0xFFFFFFFF
#define L2CAP_DEFAULT_ACC_LAT 0xFFFFFFFF
#define L2CAP_BREDR_MAX_PAYLOAD 1019
#define L2CAP_LE_MIN_MTU 23
#define L2CAP_ECRED_CONN_SCID_MAX 5

struct sockaddr_l2 {
    sa_family_t l2_family;
    __le16 l2_psm;
    bdaddr_t l2_bdaddr;
    __le16 l2_cid;
    __u8 l2_bdaddr_type;
};

#define L2CAP_OPTIONS 0x01
struct l2cap_options {
    __u16 omtu;
    __u16 imtu;
    __u16 flush_to;
    __u8 mode;
    __u8 fcs;
    __u8 max_tx;
    __u16 txwin_size;
};

#define L2CAP_CONNINFO 0x02
struct l2cap_conninfo {
    __u16 hci_handle;
    __u8 dev_class[3];
};

#define L2CAP_LM 0x03
#define L2CAP_LM_MASTER 0x0001
#define L2CAP_LM_AUTH 0x0002
#define L2CAP_LM_ENCRYPT 0x0004
#define L2CAP_LM_TRUSTED 0x0008
#define L2CAP_LM_RELIABLE 0x0010
#define L2CAP_LM_SECURE 0x0020
#define L2CAP_LM_FIPS 0x0040

#define L2CAP_COMMAND_REJ 0x01
#define L2CAP_CONN_REQ 0x02
#define L2CAP_CONN_RSP 0x03
#define L2CAP_CONF_REQ 0x04
#define L2CAP_CONF_RSP 0x05
#define L2CAP_DISCONN_REQ 0x06
#define L2CAP_DISCONN_RSP 0x07
#define L2CAP_ECHO_REQ 0x08
#define L2CAP_ECHO_RSP 0x09
#define L2CAP_INFO_REQ 0x0a
#define L2CAP_INFO_RSP 0x0b
#define L2CAP_CONN_PARAM_UPDATE_REQ 0x12
#define L2CAP_CONN_PARAM_UPDATE_RSP 0x13
#define L2CAP_LE_CONN_REQ 0x14
#define L2CAP_LE_CONN_RSP 0x15
#define L2CAP_LE_CREDITS 0x16
#define L2CAP_ECRED_CONN_REQ 0x17
#define L2CAP_ECRED_CONN_RSP 0x18
#define L2CAP_ECRED_RECONF_REQ 0x19
#define L2CAP_ECRED_RECONF_RSP 0x1a

#define L2CAP_FEAT_FLOWCTL 0x00000001
#define L2CAP_FEAT_RETRANS 0x00000002
#define L2CAP_FEAT_BIDIR_QOS 0x00000004
#define L2CAP_FEAT_ERTM 0x00000008
#define L2CAP_FEAT_STREAMING 0x00000010
#define L2CAP_FEAT_FCS 0x00000020
#define L2CAP_FEAT_EXT_FLOW 0x00000040
#define L2CAP_FEAT_FIXED_CHAN 0x00000080
#define L2CAP_FEAT_EXT_WINDOW 0x00000100
#define L2CAP_FEAT_UCD 0x00000200

#define L2CAP_FCS_NONE 0x00
#define L2CAP_FCS_CRC16 0x01
#define L2CAP_FC_SIG_BREDR 0x02
#define L2CAP_FC_CONNLESS 0x04
#define L2CAP_FC_ATT 0x10
#define L2CAP_FC_SIG_LE 0x20
#define L2CAP_FC_SMP_LE 0x40
#define L2CAP_FC_SMP_BREDR 0x80

struct l2cap_hdr {
    __le16 len;
    __le16 cid;
} __packed;
#define L2CAP_LEN_SIZE 2
#define L2CAP_HDR_SIZE 4

struct l2cap_cmd_hdr {
    __u8 code;
    __u8 ident;
    __le16 len;
} __packed;
#define L2CAP_CMD_HDR_SIZE 4

#define L2CAP_PSM_SDP 0x0001
#define L2CAP_PSM_RFCOMM 0x0003
#define L2CAP_PSM_3DSP 0x0021
#define L2CAP_PSM_IPSP 0x0023
#define L2CAP_PSM_DYN_START 0x1001
#define L2CAP_PSM_DYN_END 0xffff
#define L2CAP_PSM_AUTO_END 0x10ff
#define L2CAP_PSM_LE_DYN_START 0x0080
#define L2CAP_PSM_LE_DYN_END 0x00ff

#define L2CAP_CID_SIGNALING 0x0001
#define L2CAP_CID_CONN_LESS 0x0002
#define L2CAP_CID_ATT 0x0004
#define L2CAP_CID_LE_SIGNALING 0x0005
#define L2CAP_CID_SMP 0x0006
#define L2CAP_CID_SMP_BREDR 0x0007
#define L2CAP_CID_DYN_START 0x0040
#define L2CAP_CID_DYN_END 0xffff
#define L2CAP_CID_LE_DYN_END 0x007f

#define L2CAP_MODE_BASIC 0x00
#define L2CAP_MODE_RETRANS 0x01
#define L2CAP_MODE_FLOWCTL 0x02
#define L2CAP_MODE_ERTM 0x03
#define L2CAP_MODE_STREAMING 0x04
#define L2CAP_MODE_LE_FLOWCTL 0x80
#define L2CAP_MODE_EXT_FLOWCTL 0x81

#include "../linux/include/net/bluetooth/sco.h"
#include "../linux/include/net/bluetooth/iso.h"

#define RFCOMM_DEFAULT_MTU 127
#define RFCOMM_DEFAULT_CREDITS 7
#define RFCOMM_MAX_CREDITS 40
#define RFCOMM_SABM 0x2f
#define RFCOMM_DISC 0x43
#define RFCOMM_UA 0x63
#define RFCOMM_DM 0x0f
#define RFCOMM_UIH 0xef
#define RFCOMM_TEST 0x08
#define RFCOMM_FCON 0x28
#define RFCOMM_FCOFF 0x18
#define RFCOMM_MSC 0x38
#define RFCOMM_RPN 0x24
#define RFCOMM_RLS 0x14
#define RFCOMM_PN 0x20
#define RFCOMM_NSC 0x04

struct sockaddr_rc {
    sa_family_t rc_family;
    bdaddr_t rc_bdaddr;
    u8 rc_channel;
};

#define RFCOMM_CONNINFO 0x02
struct rfcomm_conninfo {
    __u16 hci_handle;
    __u8 dev_class[3];
};

#define RFCOMM_LM 0x03
#define RFCOMM_LM_MASTER 0x0001
#define RFCOMM_LM_AUTH 0x0002
#define RFCOMM_LM_ENCRYPT 0x0004
#define RFCOMM_LM_TRUSTED 0x0008
#define RFCOMM_LM_RELIABLE 0x0010
#define RFCOMM_LM_SECURE 0x0020
#define RFCOMM_LM_FIPS 0x0040
#define RFCOMM_MAX_DEV 256

#define BNEP_MAX_PROTO_FILTERS 5
#define BNEP_MAX_MULTICAST_FILTERS 20
#define BNEP_UUID16 0x02
#define BNEP_UUID32 0x04
#define BNEP_UUID128 0x16
#define BNEP_SVC_PANU 0x1115
#define BNEP_SVC_NAP 0x1116
#define BNEP_SVC_GN 0x1117

#define HIDP_HEADER_TRANS_MASK 0xf0
#define HIDP_HEADER_PARAM_MASK 0x0f
#define HIDP_TRANS_HANDSHAKE 0x00
#define HIDP_TRANS_HID_CONTROL 0x10
#define HIDP_TRANS_GET_REPORT 0x40
#define HIDP_TRANS_SET_REPORT 0x50
#define HIDP_TRANS_GET_PROTOCOL 0x60
#define HIDP_TRANS_SET_PROTOCOL 0x70
#define HIDP_TRANS_GET_IDLE 0x80
#define HIDP_TRANS_SET_IDLE 0x90
#define HIDP_TRANS_DATA 0xa0
#define HIDP_TRANS_DATC 0xb0
#define HIDP_PROTO_BOOT 0x00
#define HIDP_PROTO_REPORT 0x01

#include "../linux/include/net/bluetooth/hci.h"
#include "../linux/include/net/bluetooth/hci_mon.h"
#include "../linux/include/net/bluetooth/mgmt.h"
