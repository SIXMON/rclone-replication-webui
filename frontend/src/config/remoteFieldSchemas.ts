import { z } from 'zod/v4';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export type FieldType = 'text' | 'password' | 'number' | 'select';

export interface SelectOption {
  value: string;
  label: string;
}

export interface RemoteFieldDef {
  key: string;
  label: string;
  placeholder?: string;
  helpText?: string;
  type?: FieldType;
  options?: SelectOption[];
  required?: boolean;
  defaultValue?: string;
}

export interface StandardRemoteType {
  value: string;
  label: string;
  fields: RemoteFieldDef[];
}

// ---------------------------------------------------------------------------
// Standard remote type schemas
// ---------------------------------------------------------------------------

export const STANDARD_REMOTES: StandardRemoteType[] = [
  {
    value: 's3',
    label: 'S3 (Amazon, Minio, Ceph...)',
    fields: [
      {
        key: 'provider',
        label: 'Fournisseur',
        type: 'select',
        required: true,
        defaultValue: 'AWS',
        options: [
          { value: 'AWS', label: 'Amazon Web Services' },
          { value: 'Minio', label: 'Minio' },
          { value: 'Ceph', label: 'Ceph' },
          { value: 'Other', label: 'Autre' },
        ],
      },
      {
        key: 'endpoint',
        label: 'Endpoint',
        placeholder: 'https://s3.example.com',
        helpText: 'Requis pour Minio, Ceph et autres fournisseurs non-AWS',
      },
      {
        key: 'region',
        label: 'Région',
        placeholder: 'us-east-1',
      },
      {
        key: 'access_key_id',
        label: 'Access Key ID',
        required: true,
        placeholder: 'AKIAIOSFODNN7EXAMPLE',
      },
      {
        key: 'secret_access_key',
        label: 'Secret Access Key',
        type: 'password',
        required: true,
      },
      {
        key: 'acl',
        label: 'ACL',
        placeholder: 'private',
        helpText: 'Ex : private, public-read, bucket-owner-full-control',
      },
    ],
  },
  {
    value: 'sftp',
    label: 'SFTP (SSH)',
    fields: [
      {
        key: 'host',
        label: 'Hôte',
        required: true,
        placeholder: 'serveur.example.com',
      },
      {
        key: 'port',
        label: 'Port',
        type: 'number',
        defaultValue: '22',
      },
      {
        key: 'user',
        label: 'Utilisateur',
        required: true,
        placeholder: 'admin',
      },
      {
        key: 'pass',
        label: 'Mot de passe',
        type: 'password',
        helpText: 'Laisser vide si vous utilisez une clé SSH',
      },
      {
        key: 'key_file',
        label: 'Clé SSH (chemin)',
        placeholder: '/chemin/vers/id_rsa',
        helpText: 'Chemin vers le fichier de clé privée SSH',
      },
    ],
  },
  {
    value: 'ftp',
    label: 'FTP',
    fields: [
      {
        key: 'host',
        label: 'Hôte',
        required: true,
        placeholder: 'ftp.example.com',
      },
      {
        key: 'port',
        label: 'Port',
        type: 'number',
        defaultValue: '21',
      },
      {
        key: 'user',
        label: 'Utilisateur',
        required: true,
        placeholder: 'admin',
      },
      {
        key: 'pass',
        label: 'Mot de passe',
        type: 'password',
        required: true,
      },
    ],
  },
  {
    value: 'smb',
    label: 'SMB (Partage Windows)',
    fields: [
      {
        key: 'host',
        label: 'Hôte',
        required: true,
        placeholder: '192.168.1.100',
      },
      {
        key: 'port',
        label: 'Port',
        type: 'number',
        defaultValue: '445',
      },
      {
        key: 'user',
        label: 'Utilisateur',
        required: true,
      },
      {
        key: 'pass',
        label: 'Mot de passe',
        type: 'password',
        required: true,
      },
      {
        key: 'domain',
        label: 'Domaine',
        placeholder: 'WORKGROUP',
        helpText: 'Domaine Active Directory (optionnel)',
      },
    ],
  },
  {
    value: 'azureblob',
    label: 'Azure Blob Storage',
    fields: [
      {
        key: 'account',
        label: 'Nom du compte de stockage',
        required: true,
        placeholder: 'moncompte',
        helpText: 'Nom du Storage Account Azure. Laisser vide si vous utilisez une URL SAS.',
      },
      {
        key: 'key',
        label: 'Clé du compte',
        type: 'password',
        helpText: 'Clé partagée du Storage Account. Laisser vide si vous utilisez une URL SAS.',
      },
      {
        key: 'sas_url',
        label: 'URL SAS',
        placeholder: 'https://moncompte.blob.core.windows.net/conteneur?sv=...',
        helpText: 'URL SAS au niveau du compte ou du conteneur. Alternative à la clé du compte.',
      },
      {
        key: 'endpoint',
        label: 'Endpoint',
        placeholder: 'blob.core.windows.net',
        helpText: 'Laisser vide pour utiliser l\'endpoint Azure par défaut.',
      },
    ],
  },
  {
    value: 'local',
    label: 'Local (dossier local)',
    fields: [
      {
        key: 'root',
        label: 'Chemin du dossier',
        required: true,
        placeholder: '/mnt/data/backups',
      },
    ],
  },
];

// ---------------------------------------------------------------------------
// Advanced remote types (generic key-value config)
// ---------------------------------------------------------------------------

export const ADVANCED_REMOTE_TYPES: { value: string; label: string }[] = [
  { value: 'azurefiles', label: 'Azure Files' },
  { value: 'drive', label: 'Google Drive' },
  { value: 'onedrive', label: 'OneDrive' },
  { value: 'dropbox', label: 'Dropbox' },
  { value: 'b2', label: 'Backblaze B2' },
  { value: 'swift', label: 'OpenStack Swift' },
  { value: 'webdav', label: 'WebDAV' },
  { value: 'http', label: 'HTTP' },
];

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const standardMap = new Map(STANDARD_REMOTES.map((r) => [r.value, r]));

export function getStandardRemote(type: string): StandardRemoteType | undefined {
  return standardMap.get(type);
}

export function isStandardType(type: string): boolean {
  return standardMap.has(type);
}

export function buildDefaultConfig(type: string): Record<string, string> {
  const schema = getStandardRemote(type);
  if (!schema) return {};
  const defaults: Record<string, string> = {};
  for (const field of schema.fields) {
    defaults[field.key] = field.defaultValue ?? '';
  }
  return defaults;
}

// ---------------------------------------------------------------------------
// Dynamic Zod schema builder
// ---------------------------------------------------------------------------

export function buildZodSchema(remoteType: string) {
  const standard = getStandardRemote(remoteType);

  if (standard) {
    const configShape: Record<string, z.ZodType> = {};
    for (const field of standard.fields) {
      let fieldSchema: z.ZodType = z.string();

      if (field.type === 'number') {
        fieldSchema = z.string().regex(/^\d*$/, 'Doit être un nombre');
      }

      if (field.required) {
        fieldSchema = z.string().min(1, 'Requis');
        if (field.type === 'number') {
          fieldSchema = z.string().regex(/^\d+$/, 'Doit être un nombre non vide');
        }
      }

      configShape[field.key] = field.required ? fieldSchema : fieldSchema.optional().or(z.literal(''));
    }

    return z.object({
      name: z.string().min(1, 'Requis'),
      remote_type: z.string(),
      config: z.object(configShape),
      configEntries: z.array(z.object({ key: z.string(), value: z.string() })).optional(),
    });
  }

  // Advanced type — permissive config
  return z.object({
    name: z.string().min(1, 'Requis'),
    remote_type: z.string(),
    config: z.record(z.string(), z.string()).optional(),
    configEntries: z.array(z.object({ key: z.string(), value: z.string() })).optional(),
  });
}
