// #include <stdio.h>
// #include <stdlib.h>
// #include <string.h>
// #include <sysexits.h>
// #include <unistd.h>
// #include <sys/stat.h>

// struct eolstats
// {
//   long unix_eol_count;
//   long mac_eol_count;
//   long dos_eol_count;
// };

// static void usage()
// {
//   fprintf(stderr, "usage: fixle [-f] [-n] [-v] [-m | -d] files...\n");
//   fprintf(stderr, "       Fix end-of-line characters, replacing with UNIX end-of-line characters ^J\n");
//   fprintf(stderr, "       -m: use Mac-style end-of-line characters (^M)\n");
//   fprintf(stderr, "       -d: use DOS-style end-of-line characters (^M^J)\n");
//   fprintf(stderr, "       -n: don't replace lines (implies verbose)\n");
//   fprintf(stderr, "       -f: operate on files that appear binary without warning\n");
//   fprintf(stderr, "       -v: show original end-of-line character count\n");
//   exit(EX_USAGE);
// }

// int seemsBinary(FILE *f)
// {
// #define PREFIX_SIZE_TO_CHECK 2048
// #define MAX_PERCENT 3.0

//   unsigned char buffer[PREFIX_SIZE_TO_CHECK];
//   int bytesRead;
//   int nonAsciiCount = 0;
//   float percentNonAscii;
//   int i;

//   rewind(f);
//   if (f == 0) return -1;
//   bytesRead = fread(buffer, 1, PREFIX_SIZE_TO_CHECK, f);
//   if (bytesRead < 1) return 0; /* empty files are not binary */

//   for (i=0;i<bytesRead;i++) {
//     unsigned char c = buffer[i];
//     if ((c==0) || (c > 0x7f)) nonAsciiCount++;
//   }
//   percentNonAscii = 100.0*nonAsciiCount/(float)bytesRead;
//   return (percentNonAscii >= MAX_PERCENT);
// }

// int isDirectory(const char *path)
// {
//   struct stat sb;
//   if (!lstat(path, &sb)) {
//     return (S_ISDIR(sb.st_mode));
//   }
//   return 0;
// }

// static int fixLineEnds(FILE *in, FILE *out, const char *newLineEnd, struct eolstats *stats)
// {
//   int lineEndSize = strlen(newLineEnd);
//   int c;
//   int atEnd = 0;

//   stats->unix_eol_count = 0;
//   stats->mac_eol_count = 0;
//   stats->dos_eol_count = 0;

//   while (1) {
//     c = getc(in);
//     if (c == EOF) return 0;

//     if (c == 0xd) {
//       c = getc(in);
//       if (c == EOF) {
//         atEnd = 1;
//         stats->mac_eol_count++;
//         goto writeEOL;
//       }
//       if (c != 0xa) {
//         stats->mac_eol_count++;
//         c = ungetc(c, in);
//         /* we should never get an error here, but just in case... */
//         if (c < 0) return -1;
//         goto writeEOL;
//       }
//       stats->dos_eol_count++;
//       goto writeEOL;
//     }
//     else if (c == 0xa) {
//       stats->unix_eol_count++;
//       goto writeEOL;
//     }

//     if (out) { 
//       c = putc(c, out);
//       if (c < 0) return -1;
//     }
//     continue;
//   writeEOL:
//     if (out) {
//       c = fwrite(newLineEnd, lineEndSize, 1, out);
//       if (c < 0) return -1;
//     }
//     if (atEnd) return 0;
//   }

//   return 0;
// }

// static int copyFileToPath(FILE *original, char *path)
// {
//   FILE *newFile = fopen(path, "wb");
//   char buffer[1024];
//   int bytesRead, bytesWrit;
//   if (newFile == NULL) return -1;
//   while (1) {
//     bytesRead = fread(buffer, 1, 1024, original);
//     if (bytesRead == 0) break;
//     if (bytesRead < 0) return -1;
//     bytesWrit = fwrite(buffer, 1, bytesRead, newFile);
//     if (bytesWrit < bytesRead) return -1;
//   }
//   fclose(newFile);
//   return 0;
// }

use std::io::BufReader;
use std::io::BufWriter;
use std::fs::File;
use std::io::prelude::Write;
use std::io::prelude::Read;

fn main() {
    let input_path = "input_path.txt";
    let output_path = "output_path.txt";
    let output_eol = b"\r\n";
    let f_in = File::open(input_path).expect("can't open");
    let f_out = File::create(output_path).expect("can't open");

    let reader = BufReader::new(f_in);
    let mut writer = BufWriter::new(f_out);

    let mut bytes = reader.bytes().peekable();
    let mut mac_eol = 0;
    let mut dos_eol = 0;
    let mut unix_eol = 0;
    loop {
        let next_b: u8;
        next_b = match bytes.next() {
            Some(v) => v.unwrap(),
            None => break
        };

        let mut do_skip = false;
        let write_result = {
            if next_b == 0xd {
                let v = bytes.peek();
                if let Some(results) = v {
                    if let &Ok(extra_b) = results {
                        if extra_b != 0xa {
                            do_skip = true;
                            mac_eol += 1;
                        } else {
                            dos_eol += 1;
                        }
                    }
                    writer.write(output_eol)
                } else {
                    writer.write(output_eol)
                }
            } else if next_b == 0xa {
                unix_eol += 1;
                writer.write(output_eol)
            } else {
                writer.write(&[next_b])
            }
        };
        write_result.unwrap();
        if do_skip {
           bytes.next();
        };
    }
    println!("{}: {} Unix LE, {} Mac LE, {} DOS LE\n", input_path, unix_eol, mac_eol, dos_eol);

}


// int main(int argc, char *argv[])
// {
//   int ch;
//   int err;
//   int verbose = 0, doCopy = 1, operateOnBinary = 0;
//   char *path;
//   char *toolName = argv[0];
//   char *eol = "\012";
//   FILE *in, *out;
//   struct eolstats stats;
//   char tempname[20];
//   const char *TEMPLATE = "tmp.fixleXXXXXXXXXX";

//   while ((ch = getopt(argc, argv, "mdvnf")) != -1)
//     switch (ch) {
//     case 'm':
//       eol = "\015";
//       break;
//     case 'd':
//       eol = "\015\012";
//       break;
//     case 'f':
//       operateOnBinary = 1;
//       break;
//     case 'v':
//       verbose = 1;
//       break;
//     case 'n':
//       verbose = 1;
//       doCopy = 0;
//       break;
//     default:
//       usage();
//       break;
//     }

//   argc -= optind;
//   argv += optind;

//   if (argc < 1) usage();

//   while (argc-- > 0) {
//     path = *argv++;
//     if (isDirectory(path)) {
//       fprintf(stderr, "%s is a directory\n", path);
//       continue;
//     }
//     in = fopen(path, "rb");
//     if (in == 0) {
//       perror(path);
//       exit(-1);
//     }

//     out = fopen("/dev/null", "wb");
//     if (!operateOnBinary && seemsBinary(in) != 0) fprintf(stderr, "%s is a binary file\n", path);
//     else {
//       rewind(in);
//       if (doCopy) {
//         strcpy(tempname, TEMPLATE);
//         out = fdopen(mkstemp(tempname), "w+b");
//         if (out == 0) {
//           perror(tempname);
//           continue;
//         }
//         unlink(tempname);
//       }
//       fixLineEnds(in, out, eol, &stats);
//       if (verbose) printf("%s: %ld Unix LE, %ld Mac LE, %ld DOS LE\n", path, stats.unix_eol_count, stats.mac_eol_count, stats.dos_eol_count);

//       rewind(out);
//       fclose(in);
//       if (out && copyFileToPath(out, path)<0) {
//         perror(path);
//         continue;
//       }
//       fclose(out);
//     }
//   }
// }
