export interface SorterOptions {
    source_path: string;
    target_directory: string;
    copy_files: boolean;
    group_by_year: boolean;
}

export interface SortResultSummary {
    total_processed: number;
    success_count: number;
    error_count: number;
    errors: string[];
}