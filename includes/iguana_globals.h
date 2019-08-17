/******************************************************************************
 * Copyright © 2014-2018 The SuperNET Developers.                             *
 *                                                                            *
 * See the AUTHORS, DEVELOPER-AGREEMENT and LICENSE files at                  *
 * the top-level directory of this distribution for the individual copyright  *
 * holder information and the developer policies on copyright and licensing.  *
 *                                                                            *
 * Unless otherwise agreed in a custom licensing agreement, no part of the    *
 * SuperNET software, including this file may be copied, modified, propagated *
 * or distributed except according to the terms contained in the LICENSE file *
 *                                                                            *
 * Removal or modification of this copyright notice is prohibited.            *
 *                                                                            *
 ******************************************************************************/

#ifndef H_IGUANAGLOBALS_H
#define H_IGUANAGLOBALS_H

#ifdef ACTIVELY_DECLARE
#define CONDEXTERN
int32_t PANGEA_MAXTHREADS = 1,MAX_DEPTH = 100;
char *Iguana_validcommands[] =
{
    "inv2", "getdata2", "ConnectTo",
    "version", "verack", "getaddr", "addr", "inv", "getdata", "notfound", "getblocks", "getheaders", "headers", "tx", "block", "mempool", "ping", "pong",
    "reject", "filterload", "filteradd", "filterclear", "merkleblock", "alert", ""
};

#ifdef __PNACL__
char GLOBAL_TMPDIR[512] = "/tmp";
char GLOBAL_DBDIR[512] = "DB";
char GLOBAL_GENESISDIR[512] = "genesis";
char GLOBAL_HELPDIR[512] = "DB/help";
char GLOBAL_VALIDATEDIR[512] = "DB/purgeable";
char GLOBAL_CONFSDIR[512] = "DB";
int32_t IGUANA_NUMHELPERS = 1;
#else
char GLOBAL_TMPDIR[512] = "tmp";
char GLOBAL_HELPDIR[512] = "help";
char GLOBAL_DBDIR[512] = "DB";
char GLOBAL_GENESISDIR[512] = "genesis";
char GLOBAL_VALIDATEDIR[512] = "DB/purgeable";
char GLOBAL_CONFSDIR[512] = "confs";
#ifdef __linux
int32_t IGUANA_NUMHELPERS = 8;
#else
int32_t IGUANA_NUMHELPERS = 1;
#endif
#endif

#else
#define CONDEXTERN extern
#endif


// ALL globals must be here!
//CONDEXTERN struct basilisk_relay RELAYS[BASILISK_MAXRELAYS];
//CONDEXTERN int32_t NUMRELAYS,RELAYID;

CONDEXTERN char *COMMANDLINE_ARGFILE;
CONDEXTERN char *Iguana_validcommands[];
CONDEXTERN int32_t Showmode,Autofold,PANGEA_MAXTHREADS,QUEUEITEMS;

CONDEXTERN struct gecko_chain *Categories;
//CONDEXTERN struct iguana_info *Allcoins;
CONDEXTERN char Userhome[512];
CONDEXTERN int32_t FIRST_EXTERNAL,IGUANA_disableNXT,Debuglevel,IGUANA_BIGENDIAN;
CONDEXTERN uint32_t prices777_NXTBLOCK;
CONDEXTERN queue_t helperQ,JSON_Q,FINISHED_Q,bundlesQ,emitQ;
CONDEXTERN struct supernet_info MYINFO,**MYINFOS;
CONDEXTERN int32_t MAIN_initflag,MAX_DEPTH;
CONDEXTERN int32_t HDRnet,netBLOCKS;
CONDEXTERN cJSON *API_json;

CONDEXTERN char GLOBAL_TMPDIR[512];
CONDEXTERN char GLOBAL_DBDIR[512];
CONDEXTERN char GLOBAL_GENESISDIR[512];
CONDEXTERN char GLOBAL_HELPDIR[512];
CONDEXTERN char GLOBAL_VALIDATEDIR[512];
CONDEXTERN char GLOBAL_CONFSDIR[512];
CONDEXTERN int32_t IGUANA_NUMHELPERS;

#define CRYPTO777_PUBSECPSTR "02d8e32a6e677f862ea6d52006ab8fc2a72c96b7d7eff4921db1c58714a2339dbe"
#define CRYPTO777_RMD160STR "BE32CD9D97A9A532BE370D3559E113D9CC915C83"
#define CRYPTO777_BTCADDR "mxrdYNMi2QVSizeRDottwjxV1pFqPWPWf1"
#define CRYPTO777_KMDADDR "RScsKqA1pCrm1tXzyQueDM5Mv67jAh54oz"

CONDEXTERN uint8_t CRYPTO777_RMD160[20],CRYPTO777_PUBSECP33[33];

#endif

