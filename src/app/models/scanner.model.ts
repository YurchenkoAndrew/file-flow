export type FileCategory =
    | 'Images'
    | 'Videos'
    | 'Documents'
    | 'Audios'
    | 'Archives'
    | 'Code'
    | 'Software'
    | 'Mobile'
    | 'DiskImages'
    | 'Fonts'
    | 'DesignProjects'
    | 'VideoProjects'
    | 'Other';

export interface FileItem {
    path: string;
    name: string;
    extension: string;
    size: number;
    category: FileCategory;
}

export interface CategoryStat {
    category: FileCategory;
    total_size: number;
    files_count: number;
    percentage: number;
}

export interface DuplicateGroup {
    size: number;
    files: FileItem[];
}

export interface ScanResultSummary {
    session_id: number;
    total_size: number;
    total_files_count: number;
    category_stats: CategoryStat[];
    largest_files: FileItem[];
    duplicates_estimated_size: number;
    duplicate_groups: DuplicateGroup[];
}