
export type ExtensionMetadata={
    name: string;
    developers: string[];
    icon: string | null;
    description: string;
    tags: string[];
    app: string
}

export type ManagedExtensionMetadata = {
    downloads: number;
    latest: {
        release: string | null;
        beta: string | null;
        rc: string | null;
    };
    versions: Array<{
        version: string;
        release_type: string;
        metadata_path: string;
    }>;
};

export enum ExtensionState {
    Enabled,
    Disabled,
}

export type WrappedExtension = {
    metadata: ExtensionMetadata,
    state: ExtensionState,
    pointer: ExtensionPointer,
}

export type SearchResult = {
    result: [
        {
            group: string,
            name: string
        }
    ]
}

export enum RepositoryType {
    REMOTE, LOCAL
}

export type ExtensionPointer = {
    descriptor: string,
    repository: string,
    repository_type: RepositoryType
}