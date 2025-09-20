#include <fltKernel.h>
#include <ntddk.h>

#define POOL_TAG		'DroC'

static const UNICODE_STRING blacklist[] = {
	RTL_CONSTANT_STRING(L"mimikatz.exe"),

	RTL_CONSTANT_STRING(L"winpeas.bat"),
	RTL_CONSTANT_STRING(L"winpeas.ps1"),
	RTL_CONSTANT_STRING(L"winpeas.exe"),
	RTL_CONSTANT_STRING(L"winpeasx64.exe"),
	RTL_CONSTANT_STRING(L"winpeasx86.exe"),
	RTL_CONSTANT_STRING(L"winpeasany.exe"),
	RTL_CONSTANT_STRING(L"winpeasany_ofs.exe"),

	RTL_CONSTANT_STRING(L"bloodhound.exe"),
	RTL_CONSTANT_STRING(L"bloodhound.ps1"),

	RTL_CONSTANT_STRING(L"seatbelt.exe"),
	RTL_CONSTANT_STRING(L"seatbelt64.exe"),
};

PFLT_FILTER gFilterHandle = NULL;
UINT32* pids = NULL;
size_t pidsCapacity = 0, pidsCount = 0;
KMUTEX pidsMutex;

NTSTATUS DriverUnload(_In_ FLT_FILTER_UNLOAD_FLAGS flags);
FLT_PREOP_CALLBACK_STATUS PreOperationCallback(
	PFLT_CALLBACK_DATA Data,
	PCFLT_RELATED_OBJECTS FltObjects,
	PVOID* CompletionContext
);

const FLT_OPERATION_REGISTRATION Callbacks[] = {
	{
		IRP_MJ_WRITE,
		0,
		PreOperationCallback,
		NULL
	},
	{
		IRP_MJ_OPERATION_END
	}
};

const FLT_REGISTRATION FilterRegistration = {
	sizeof(FLT_REGISTRATION),
	FLT_REGISTRATION_VERSION,
	0,
	NULL,
	Callbacks,
	DriverUnload,
	NULL, NULL, NULL, NULL, NULL, NULL
};

BOOLEAN unicodeContains(
	const UNICODE_STRING haystack,
	const UNICODE_STRING needle
) {
	if (needle.Length > haystack.Length) {
		return FALSE;
	}

	for (USHORT i = 0; i <= haystack.Length - needle.Length; i += sizeof(WCHAR)) {
		if (RtlCompareMemory(
			(PUCHAR)haystack.Buffer + i,
			needle.Buffer,
			needle.Length) == needle.Length) {
			return TRUE;
		}
	}

	return FALSE;
}

// TODO: finish logging
void handleLog() {
	if (pids == NULL || pidsCount == 0) {
		return;
	}

	DbgPrint("[CorDefender] Log here\n");
}

void addPid(UINT32 pid) {
	UINT32* newPids;
	size_t initialCapacity;

	KeWaitForMutexObject(&pidsMutex, Executive, KernelMode, FALSE, NULL);

	if (pids == NULL) {
		initialCapacity = 64;
		pids = ExAllocatePool2(POOL_FLAG_NON_PAGED, initialCapacity * sizeof(UINT32), POOL_TAG);
		if (pids == NULL) {
			KeReleaseMutex(&pidsMutex, FALSE);
			return;
		}

		pidsCapacity = initialCapacity;
	}

	for (UINT32 i = 0; i < pidsCount; i++) {
		if (pids[i] == pid) {
			KeReleaseMutex(&pidsMutex, FALSE);
			return;
		}
	}

	pids[pidsCount] = pid;

	if (pidsCount++ >= pidsCapacity) {
		newPids = ExAllocatePool2(
			POOL_FLAG_NON_PAGED,
			(pidsCapacity * 2) * sizeof(UINT32),
			POOL_TAG
		);
		if (newPids == NULL) {
			KeReleaseMutex(&pidsMutex, FALSE);
			return;
		}

		RtlCopyMemory(newPids, pids, (pidsCount - 1) * sizeof(UINT32));
		ExFreePoolWithTag(pids, POOL_TAG);

		pids = newPids;
		pidsCapacity *= 2;
	}

	KeReleaseMutex(&pidsMutex, FALSE);
}

FLT_PREOP_CALLBACK_STATUS PreOperationCallback(
	PFLT_CALLBACK_DATA Data,
	PCFLT_RELATED_OBJECTS FltObjects,
	PVOID* CompletionContext
) {
	UNREFERENCED_PARAMETER(FltObjects);
	UNREFERENCED_PARAMETER(CompletionContext);

	PAGED_CODE();

	PFLT_FILE_NAME_INFORMATION filenameInfo;
	NTSTATUS status;
	UINT32 pid;
	UNICODE_STRING lowercase;

	if (Data->Iopb->MajorFunction != IRP_MJ_WRITE) {
		return FLT_PREOP_SUCCESS_NO_CALLBACK;
	}

	status = FltGetFileNameInformation(
		Data,
		FLT_FILE_NAME_NORMALIZED,
		&filenameInfo
	);
	if (!NT_SUCCESS(status)) {
		return FLT_PREOP_SUCCESS_NO_CALLBACK;
	}

	RtlInitUnicodeString(&lowercase, NULL);
	FltParseFileNameInformation(filenameInfo);
	status = RtlDowncaseUnicodeString(&lowercase, &filenameInfo->Name, TRUE);
	if (!NT_SUCCESS(status)) {
		RtlFreeUnicodeString(&lowercase);
		FltReleaseFileNameInformation(filenameInfo);
		return FLT_PREOP_SUCCESS_NO_CALLBACK;
	}

	for (UINT32 i = 0; i < sizeof(blacklist) / sizeof(blacklist[0]); i++) {
		if (unicodeContains(lowercase, blacklist[i])) {
			Data->IoStatus.Status = STATUS_ACCESS_DENIED;
			Data->IoStatus.Status = 0;

			RtlFreeUnicodeString(&lowercase);
			FltReleaseFileNameInformation(filenameInfo);

			pid = (UINT32)PtrToUlong(PsGetCurrentProcessId());
			addPid(pid);

			return FLT_PREOP_COMPLETE;
		}
	}

	RtlFreeUnicodeString(&lowercase);
	FltReleaseFileNameInformation(filenameInfo);

	return FLT_PREOP_SUCCESS_NO_CALLBACK;
}

NTSTATUS DriverUnload(_In_ FLT_FILTER_UNLOAD_FLAGS flags) {
	UNREFERENCED_PARAMETER(flags);

	PAGED_CODE();

	FltUnregisterFilter(gFilterHandle);

	if (pids != NULL) {
		ExFreePoolWithTag(pids, POOL_TAG);
	}

	return STATUS_SUCCESS;
}

NTSTATUS DriverEntry(
	_In_ PDRIVER_OBJECT driverObject,
	_In_ PUNICODE_STRING registryPath
) {
	UNREFERENCED_PARAMETER(registryPath);

	NTSTATUS status;

	status = FltRegisterFilter(
		driverObject,
		&FilterRegistration,
		&gFilterHandle
	);
	if (!NT_SUCCESS(status)) {
		return status;
	}

	status = FltStartFiltering(gFilterHandle);
	if (!NT_SUCCESS(status)) {
		FltUnregisterFilter(gFilterHandle);
		return status;
	}

	KeInitializeMutex(&pidsMutex, 0);

	return STATUS_SUCCESS;
}