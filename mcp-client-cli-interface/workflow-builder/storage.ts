// ... existing code ...
class WorkflowStorage {
    private storageLocation: string;
    
    constructor(storageLocation = `${process.env.HOME}/.screenpipe/workflows`) {
      this.storageLocation = storageLocation;
      this.ensureStorageExists();
    }
    
    private ensureStorageExists() {
      if (!fs.existsSync(this.storageLocation)) {
        fs.mkdirSync(this.storageLocation, { recursive: true });
        console.log(`created workflow storage at ${this.storageLocation}`);
      }
    }
    
    async saveWorkflow(workflow: WorkflowDefinition, name: string): Promise<string> {
      // Add name to metadata if not present
      if (!workflow.metadata.name) {
        workflow.metadata.name = name;
      }
      
      const filename = `${name.replace(/\s+/g, '-').toLowerCase()}-${Date.now()}.json`;
      const filepath = path.join(this.storageLocation, filename);
      
      await fs.promises.writeFile(
        filepath,
        JSON.stringify(workflow, null, 2),
        'utf8'
      );
      
      console.log(`saved workflow to ${filepath}`);
      return filepath;
    }
    
    async loadWorkflow(nameOrPath: string): Promise<WorkflowDefinition> {
      let filepath = nameOrPath;
      
      // If it doesn't look like a path, treat as a name
      if (!nameOrPath.includes('/') && !nameOrPath.includes('\\')) {
        const files = await fs.promises.readdir(this.storageLocation);
        const matchingFile = files.find(f => f.startsWith(nameOrPath.replace(/\s+/g, '-').toLowerCase()));
        
        if (!matchingFile) {
          throw new Error(`No workflow found with name: ${nameOrPath}`);
        }
        
        filepath = path.join(this.storageLocation, matchingFile);
      }
      
      const content = await fs.promises.readFile(filepath, 'utf8');
      return JSON.parse(content) as WorkflowDefinition;
    }
    
    async listWorkflows(): Promise<WorkflowSummary[]> {
      const files = await fs.promises.readdir(this.storageLocation);
      
      const workflows: WorkflowSummary[] = [];
      
      for (const file of files) {
        if (file.endsWith('.json')) {
          try {
            const filepath = path.join(this.storageLocation, file);
            const content = await fs.promises.readFile(filepath, 'utf8');
            const workflow = JSON.parse(content) as WorkflowDefinition;
            
            workflows.push({
              name: workflow.metadata.name || file.replace('.json', ''),
              description: workflow.metadata.description,
              createdAt: workflow.metadata.createdAt,
              stepCount: workflow.steps.length,
              filename: file,
              filepath
            });
          } catch (err) {
            console.error(`error reading workflow file ${file}:`, err);
          }
        }
      }
      
      return workflows;
    }
  }
  // ... existing code ...