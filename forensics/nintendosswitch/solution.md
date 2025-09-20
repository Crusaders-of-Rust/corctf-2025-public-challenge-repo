Nintendo SSwitch

```bash
# cat /etc/nsswitch.conf 
passwd: files ctf
shadow: files
hosts:  files dns
```

```bash
# ltrace getent passwd nobody  
mtrace()                                                                                                                          = <void>
setlocale(LC_ALL, "")                                                                                                             = "C"
textdomain("libc")                                                                                                                = "libc"
argp_parse(0x5e9e9aef5440, 3, 0x7fff97ae1d28, 0)                                                                                  = 0
strcmp("passwd", "passwd")                                                                                                        = 0
__errno_location()                                                                                                                = 0x7db7422c36c0
strtoul(0x7fff97ae3f27, 0x7fff97ae1b70, 10, 176)                                                                                  = 0
getpwnam("nobody")                                                                                                                = 0x7db74249f9c0
putpwent(0x7db74249f9c0, 0x7db742499760, 0, 1nobody:x:65534:65534:nobody:/nonexistent:/usr/sbin/nologin
)                                                                                    = 0
+++ exited (status 0) +++
```

```bash
# ltrace getent passwd testuser
mtrace()                                                                                                                          = <void>
setlocale(LC_ALL, "")                                                                                                             = "C"
textdomain("libc")                                                                                                                = "libc"
argp_parse(0x5eb7ffeae440, 3, 0x7ffd8cc723c8, 0)                                                                                  = 0
strcmp("passwd", "passwd")                                                                                                        = 0
__errno_location()                                                                                                                = 0x71cc7281e6c0
strtoul(0x7ffd8cc72f25, 0x7ffd8cc72210, 10, 176)                                                                                  = 0
getpwnam("testuser")                                                                                                              = 0
+++ exited (status 2) +++
```

```bash
# ls /lib/x86_64-linux-gnu/ | grep nss
libjansson.so.4
libjansson.so.4.14.0
libnss_compat.so.2
libnss_ctf.so.2
libnss_dns.so.2
libnss_files.so.2
libnss_hesiod.so.2
```

```bash
# strings libnss_ctf.so.2 
...
...
...
get_blob_path
get_outfile
get_trigger_user
...
...
...
strcmp
...
...
...
```

```bash
# objdump -d /lib/x86_64-linux-gnu/libnss_ctf.so.2
...
...
...
0000000000001298 <get_trigger_user@@Base>:
    1298:	55                   	push   %rbp
    1299:	48 89 e5             	mov    %rsp,%rbp
    129c:	48 b8 11 16 11 0b 1a 	movabs $0x101b111a0b111611,%rax
    12a3:	11 1b 10 
    12a6:	48 89 45 f3          	mov    %rax,-0xd(%rbp)
    12aa:	c6 45 fb 7f          	movb   $0x7f,-0x5(%rbp)
    12ae:	c7 45 fc 00 00 00 00 	movl   $0x0,-0x4(%rbp)
    12b5:	eb 22                	jmp    12d9 <get_trigger_user@@Base+0x41>
    12b7:	8b 45 fc             	mov    -0x4(%rbp),%eax
    12ba:	48 98                	cltq
    12bc:	0f b6 44 05 f3       	movzbl -0xd(%rbp,%rax,1),%eax
    12c1:	83 f0 7f             	xor    $0x7f,%eax
    12c4:	89 c1                	mov    %eax,%ecx
    12c6:	8b 45 fc             	mov    -0x4(%rbp),%eax
    12c9:	48 98                	cltq
    12cb:	48 8d 15 e6 2d 00 00 	lea    0x2de6(%rip),%rdx        # 40b8 <_nss_ctf_getpwnam_r@@Base+0x2bfc>
    12d2:	88 0c 10             	mov    %cl,(%rax,%rdx,1)
    12d5:	83 45 fc 01          	addl   $0x1,-0x4(%rbp)
    12d9:	8b 45 fc             	mov    -0x4(%rbp),%eax
    12dc:	83 f8 08             	cmp    $0x8,%eax
    12df:	76 d6                	jbe    12b7 <get_trigger_user@@Base+0x1f>
    12e1:	48 8d 05 d0 2d 00 00 	lea    0x2dd0(%rip),%rax        # 40b8 <_nss_ctf_getpwnam_r@@Base+0x2bfc>
    12e8:	5d                   	pop    %rbp
    12e9:	c3                   	ret
...
...
...
```

| Byte | XOR 0x7f | ASCII |
| ---- | -------- | ----- |
| 0x11 | 0x6e     | **n** |
| 0x16 | 0x69     | **i** |
| 0x11 | 0x6e     | **n** |
| 0x0b | 0x74     | **t** |
| 0x1a | 0x65     | **e** |
| 0x11 | 0x6e     | **n** |
| 0x1b | 0x64     | **d** |
| 0x10 | 0x6f     | **o** |

`get_trigger_user` = **"nintendo"**

```bash
# strace getent passwd nintendo
...
...
...
openat(AT_FDCWD, "/tmp/flag.txt", O_WRONLY|O_CREAT|O_TRUNC, 0666) = 3
newfstatat(3, "", {st_mode=S_IFREG|0644, st_size=0, ...}, AT_EMPTY_PATH) = 0
write(3, "corctf{nsswitch_can_be_sneaky_so"..., 52) = 52
close(3)                                = 0
exit_group(2)                           = ?
+++ exited with 2 +++
```

```bash
# cat /tmp/flag.txt 
corctf{nsswitch_can_be_sneaky_sometimes_i_guess_idk}root@4ae50e011d89:~# 

```
