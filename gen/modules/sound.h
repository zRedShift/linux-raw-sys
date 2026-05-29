// ALSA/sound userspace API headers.

#include "support.h"

// Ensure the kernel-style include path is used instead of the userspace path,
// which may not resolve under -nostdinc on all architectures.
#ifndef __linux__
#define __linux__
#endif

#ifdef __m68k__
#pragma pack(push, 2)
#endif
#include <sound/asound.h>
#ifdef __m68k__
#pragma pack(pop)
#endif
