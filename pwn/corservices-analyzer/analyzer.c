#include <stdio.h>
#include <windows.h>
#include <math.h>
#include <time.h>

#define MAX_FILE_SIZE 1024 * 1024 * 20
#define ENTROPY_TRESHOLD 7.5

const char *antiDebugSymbols[] = {
    "IsDebuggerPresent",
    "CheckRemoteDebuggerPresent",
    "OutputDebugString",
    "FindWindow",
    "NtQueryInformationProcess",
    "NtSetInformationThread",
};

float calculateEntropy(PBYTE bytes, DWORD size) {
    UINT freq[256] = {};
    float entropy = 0.0;
    float p;

    if (size == 0) {
        return 0.0;
    }

    for (UINT i = 0; i < size; i++) {
        freq[bytes[i]]++;
    }

    for (UINT i = 0; i < 256; i++) {
        if (freq[i] == 0) {
            continue;
        }

        p = (float)freq[i] / size;
        entropy -= p * log2(p);
    }

    return entropy;
}

PBYTE readFile(const char *filename, size_t *fileSize) {
    FILE *file;
    PBYTE fileBytes;

    file = fopen(filename, "rb");
    if (file == NULL) {
        perror("[E] fopen");
        return NULL;
    }

    fseek(file, 0, SEEK_END);
    *fileSize = ftell(file);
    fseek(file, 0, SEEK_SET);

    if (*fileSize > MAX_FILE_SIZE) {
        printf("[E] File too large\n");
        return NULL;
    }

    fileBytes = HeapAlloc(GetProcessHeap(), HEAP_ZERO_MEMORY, *fileSize);
    if (fileBytes == NULL) {
        printf("[E] No memory\n");
        return NULL;
    }

    if (fread(fileBytes, *fileSize, 1, file) <= 0) {
        perror("[E] fread");
        return NULL;
    }

    fclose(file);
    return fileBytes;
}

BOOL isValidPE(
        PBYTE file,
        size_t fileSize
) {
    PIMAGE_DOS_HEADER dosHeader = (PIMAGE_DOS_HEADER)file;
    PIMAGE_NT_HEADERS ntHeader;

    if (sizeof(*dosHeader) > fileSize) {
        return FALSE;
    }

    if (dosHeader->e_magic != IMAGE_DOS_SIGNATURE) {
        return FALSE;
    }

    if (
        (dosHeader->e_lfanew + sizeof(*ntHeader) < dosHeader->e_lfanew) ||
        (dosHeader->e_lfanew + sizeof(*ntHeader) > fileSize)
    ) {
        return FALSE;
    }

    ntHeader = (PIMAGE_NT_HEADERS)(file + dosHeader->e_lfanew);
    if (ntHeader->Signature != IMAGE_NT_SIGNATURE) {
        return FALSE;
    }

    if (
        (ntHeader->OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR32_MAGIC) &&
        (ntHeader->OptionalHeader.Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC)
    ) {
        return FALSE;
    }

    return TRUE;
}

void printBasic(PIMAGE_NT_HEADERS ntHeader) {
    time_t timestamp = ntHeader->FileHeader.TimeDateStamp;

    printf("[i] Basic information:\n");
    printf(
        "\tMachine: %s (0x%x)\n",
        ntHeader->FileHeader.Machine == 0x8664 ? "x64" : "x86",
        ntHeader->FileHeader.Machine
    );
    printf(
        "\tType: %s\n",
        ntHeader->FileHeader.Characteristics & IMAGE_FILE_DLL ? "DLL" : "EXE"
    );
    printf("\tTimestamp: %s", ctime(&timestamp));
    printf("\tImage Base: 0x%llx\n", ntHeader->OptionalHeader.ImageBase);
    printf("\tEntry Point: 0x%lx\n", ntHeader->OptionalHeader.AddressOfEntryPoint);
    printf("\tBase of Code: 0x%lx\n", ntHeader->OptionalHeader.BaseOfCode);
    printf("\n");
}

void printSection(
        PIMAGE_NT_HEADERS ntHeader,
        PBYTE file,
        size_t fileSize,
        PIMAGE_SECTION_HEADER *importSection
) {
    short sectionNum = ntHeader->FileHeader.NumberOfSections;
    IMAGE_DATA_DIRECTORY importDir = ntHeader->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    PIMAGE_SECTION_HEADER sectionHeader = IMAGE_FIRST_SECTION(ntHeader);
    
    if (sectionNum < 512) {
        float entropy;
        UINT heCount = 0;
        char highEntropies[IMAGE_SIZEOF_SHORT_NAME][sectionNum];

        printf("[i] Section information:\n");
        printf("\tSTART\t\tSIZE\t\tENTROPY\t\tNAME\n");
        for (UINT i = 0; i < sectionNum; i++) { 
            if ((PBYTE)&sectionHeader[i] > file + fileSize - sizeof(IMAGE_SECTION_HEADER)) {
                break;
            }
            
            if (
                (sectionHeader[i].PointerToRawData + sectionHeader[i].SizeOfRawData < sectionHeader[i].PointerToRawData) ||
                (sectionHeader[i].PointerToRawData + sectionHeader[i].SizeOfRawData > fileSize)
            ) {
                break;
            }

            entropy = calculateEntropy(
                (PBYTE)(file + sectionHeader[i].PointerToRawData),
                sectionHeader[i].SizeOfRawData
            );

            printf(
                "\t%lx\t\t%lx\t\t%.2f\t\t%.8s",
                sectionHeader[i].PointerToRawData,
                sectionHeader[i].SizeOfRawData,
                entropy,
                sectionHeader[i].Name
            );

            if (entropy >= ENTROPY_TRESHOLD) {
                printf("\t\t[!] High entropy\n");
                strncpy_s(
                    (char *)(highEntropies) + heCount++ * IMAGE_SIZEOF_SHORT_NAME,
                    IMAGE_SIZEOF_SHORT_NAME,
                    sectionHeader[i].Name,
                    _TRUNCATE
                );
            } else {
                printf("\n");
            }

            if (
                (importDir.VirtualAddress >= sectionHeader[i].VirtualAddress) &&
                (importDir.VirtualAddress < sectionHeader[i].VirtualAddress + sectionHeader[i].Misc.VirtualSize)
            ) {
                *importSection = &sectionHeader[i];
            }
        }

        printf("\n");

        if (heCount > 0) {
            printf("[!] Sections ");
            for (UINT i = 0; i < heCount; i++) {
                printf("%.8s", (char *)(highEntropies) + i * IMAGE_SIZEOF_SHORT_NAME);

                if (i != heCount - 1) {
                    printf(", ");
                }
            }
            printf(" have high entropies, possible obfuscation\n");
            
            printf("\n");
        }
    } else {
        printf("[i] Too many sections (%d), skipping section information", sectionNum);
    }
}

void printImports(
        PIMAGE_NT_HEADERS ntHeader,
        PBYTE file,
        size_t fileSize,
        PIMAGE_SECTION_HEADER importSection
) {
    DWORD64 offset = (DWORD64)file + importSection->PointerToRawData;
    IMAGE_DATA_DIRECTORY importDir = ntHeader->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    PIMAGE_IMPORT_DESCRIPTOR importDesc = (PIMAGE_IMPORT_DESCRIPTOR)(
        offset + (importDir.VirtualAddress - importSection->VirtualAddress)
    );
    PIMAGE_THUNK_DATA thunkData;
    DWORD thunk;
    PBYTE dll, sym;
    BOOL antiDebug = FALSE;

    if (
        ((PBYTE)importDesc < file) ||
        ((PBYTE)importDesc > file + fileSize - sizeof(IMAGE_IMPORT_DESCRIPTOR))
    ) {
        printf("[E] Have invalid import descriptor address, corrupted PE?\n");
        return;
    }

    printf("[i] Import information:\n");
    while (
        ((PBYTE)importDesc <= file + fileSize - sizeof(IMAGE_IMPORT_DESCRIPTOR)) &&
        (importDesc->Name != 0)
    ) {
        dll = (PBYTE)(offset + (importDesc->Name - importSection->VirtualAddress));
        if (dll < file || dll >= file + fileSize) {
            break;
        }

        thunk = importDesc->OriginalFirstThunk == 0 ?
            importDesc->FirstThunk : importDesc->OriginalFirstThunk;
        thunkData = (PIMAGE_THUNK_DATA)(offset + (thunk - importSection->VirtualAddress));
        if (
            ((PBYTE)thunkData < file) ||
            ((PBYTE)thunkData > file + fileSize - sizeof(IMAGE_THUNK_DATA))
        ) {
            printf("[E] Have invalid thunk data address, corrupted PE?\n");
            break;
        }

        printf(" - %s:\n", dll);
        while (
            ((PBYTE)thunkData <= file + fileSize - sizeof(IMAGE_THUNK_DATA)) &&
            (thunkData->u1.AddressOfData != 0)
        ) {
            sym = (PBYTE)(offset + (thunkData->u1.AddressOfData - importSection->VirtualAddress + 2));
            if (sym < file || sym >= file + fileSize) {
                break;
            }

            if (strcmp((char *)dll, "ntdll.dll") != 0) {
                printf("\t%s\n", sym);
                thunkData++;
                continue;
            }

            printf("\t%s", sym);
            for (UINT i = 0; i < sizeof(antiDebugSymbols) / sizeof(antiDebugSymbols[0]); i++) {
                if (strcmp(antiDebugSymbols[i], (char *)sym) == 0) {
                    antiDebug = TRUE;
                    printf("\t\t[!] Sometimes used for anti-debugging");
                    break;
                }
            }

            printf("\n");
            thunkData++;
        }

        importDesc++;
    }

    printf("\n");

    if (antiDebug) {
        printf("[!] Imports contain functions used for anti-debugging techniques\n");
    }

    printf("\n");
}

void runScan(PBYTE file, size_t fileSize) {
    PIMAGE_DOS_HEADER dosHeader;
    PIMAGE_NT_HEADERS ntHeader;
    PIMAGE_SECTION_HEADER importSection = NULL;

    if (isValidPE(file, fileSize) == FALSE) {
        printf("[E] Invalid PE file\n");
        return;
    }

    dosHeader = (PIMAGE_DOS_HEADER)file;
    ntHeader = (PIMAGE_NT_HEADERS)(file + dosHeader->e_lfanew);

    printBasic(ntHeader);
    printSection(ntHeader, file, fileSize, &importSection);

    if (importSection != NULL) {
        printImports(ntHeader, file, fileSize, importSection);
    } else {
        printf("[i] Empty import table\n");
    }
}

int main(int argc, const char **argv) {
    PBYTE file;
    size_t fileSize;

    if (argc != 2) {
        printf("Usage: %s <file>\n", argv[0]);
        return 1;
    }

    file = readFile(argv[1], &fileSize);
    if (file == NULL) {
        return 1;
    }

    runScan(file, fileSize);

    HeapFree(GetProcessHeap(), 0, file);
    return 0;
}
