# adobe_telemetry_block_rs

```
// crontab every 30 minutes
//
// FLOW START
//
// if !is_root
//      FLOW END
//
// if USER_DEFINED_HOSTS exists, read it and assign to PREPEND_HOSTS
// else, write the PREPEND_HOSTS into USER_DEFINED_HOSTS
//
// fetch lists.txt
//
// if the TMP_HOSTS_FILE exists, compare the existing list.txt
//      if equals, do nothing, EARLY RETURN
//            FLOW END
//      else, backup HOST_FILE(user might use it first time),
//            overwrite into TMP_HOSTS_FILE with lists.txt
// else, write into TMP_HOSTS_FILE
//
// backup the existing HOST_FILE to BACKUP_HOSTS_FILE and TMP_HOSTS_FILE to BACKUP_TMP_HOSTS_FILE
//
// write PREPEND_HOSTS and TMP_HOSTS_FILE(which will be lists.txt in memory) into
// HOST_FILE(/etc/hosts might need privilege)
//
// syscall for clear DNS
//
// FLOW END
```
