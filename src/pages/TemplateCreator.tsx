import React, { useState } from 'react';
import { invoke } from "@tauri-apps/api/core";
import '../styles/TemplateCreator.css';

type TemplateType = 'marketing' | 'utility';
type HeaderType = 'image' | 'text' | 'none';
type CreationMode = 'single' | 'all';

interface BulkResult {
  successful: number;
  total: number;
  app_ids: string[];
}

const TemplateCreator: React.FC = () => {
  const [templateName, setTemplateName] = useState('');
  const [templateType, setTemplateType] = useState<TemplateType>('marketing');
  const [headerType, setHeaderType] = useState<HeaderType>('none');
  const [headerText, setHeaderText] = useState('');
  const [headerImage, setHeaderImage] = useState<string | null>(null);
  const [bodyText, setBodyText] = useState('');
  const [imageFile, setImageFile] = useState<File | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [appId, setAppId] = useState('');
  const [vertical, setVertical] = useState('Template');
  const [creationMode, setCreationMode] = useState<CreationMode>('single');
  const [bulkResult, setBulkResult] = useState<BulkResult | null>(null);
  const [isProcessing, setIsProcessing] = useState<boolean>(false);

  const handleImageUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      setImageFile(file);
      const reader = new FileReader();
      reader.onload = (event) => {
        if (event.target?.result) {
          setHeaderImage(event.target.result as string);
        }
      };
      reader.onerror = () => {
        console.error("Erro lendo arquivo");
      };
      reader.readAsDataURL(file);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!templateName.trim()) {
      alert("Por favor insira o nome do template");
      return;
    }
    
    if (headerType === 'text' && !headerText.trim()) {
      alert("Por favor insira o texto do header");
      return;
    }
    
    if (headerType === 'image' && !headerImage) {
      alert("Por favor selecione uma imagem");
      return;
    }
    
    if (!bodyText.trim()) {
      alert("Por favor adicione o corpo da mensagem");
      return;
    }
    
    if (creationMode === 'single' && !appId.trim()) {
      alert("Por favor insira o App ID");
      return;
    }
    
    try {
      setIsSubmitting(true);
      setBulkResult(null);
      
      let imageData = null;
      let imageName = null;
      
      if (headerType === 'image' && imageFile) {
        const arrayBuffer = await imageFile.arrayBuffer();
        imageData = Array.from(new Uint8Array(arrayBuffer));
        imageName = imageFile.name;
      }
      
      if (creationMode === 'single') {
        await invoke('create_template', {
          params: {
            template_name: templateName,
            app_id: appId,
            content: bodyText,
            category: templateType.toUpperCase(),
            template_type: headerType === 'image' ? 'IMAGE' : 'TEXT',
            vertical,
            header_text: headerType === 'text' ? headerText : undefined,
            image_data: headerType === 'image' ? imageData : undefined,
            image_name: headerType === 'image' ? imageName : undefined
          }
        });
        
        alert("Template criado com sucesso!");
      } else {
        setIsProcessing(true);
        
        const result = await invoke<BulkResult>('create_template_for_all_connections', {
          params: {
            template_name: templateName,
            content: bodyText,
            category: templateType.toUpperCase(),
            template_type: headerType === 'image' ? 'IMAGE' : 'TEXT',
            vertical,
            header_text: headerType === 'text' ? headerText : undefined,
            image_data: headerType === 'image' ? imageData : undefined,
            image_name: headerType === 'image' ? imageName : undefined
          }
        });
        
        setBulkResult(result);
        alert(`Templates criados com sucesso para ${result.successful} de ${result.total} conexões.`);
      }
      
    } catch (error) {
      console.error('Error creating template:', error);
      alert(`Erro ao criar template: ${error}`);
    } finally {
      setIsSubmitting(false);
      setIsProcessing(false);
    }
  };

  const formatPreviewText = (text: string) => {
    // Replace variables with example values and convert newlines to <br> tags for HTML display
    return text
      .replace(/\{\{name\}\}/g, "John")
      .replace(/\{\{date\}\}/g, new Date().toLocaleDateString())
      .split('\n').map((line, index) => (
        <React.Fragment key={index}>
          {line}
          {index < text.split('\n').length - 1 && <br />}
        </React.Fragment>
      ));
  };

  return (
    <div className="template-creator">
      <div className="template-creator__container">
        <div className="template-creator__form-section">
          <h1>Criar novo Template</h1>
          <form onSubmit={handleSubmit}>
            <div className="form-group">
              <h2>Nome do Template</h2>
              <div className="template-name-input">
                <input
                  type="text"
                  placeholder="Digite o nome do template"
                  value={templateName}
                  onChange={(e) => setTemplateName(e.target.value)}
                  required
                />
              </div>
            </div>

            <div className="form-group">
              <h2>Modo de Criação</h2>
              <div className="creation-mode-selector">
                <button
                  type="button"
                  className={`mode-button ${creationMode === 'single' ? 'active' : ''}`}
                  onClick={() => setCreationMode('single')}
                >
                  Template Único
                </button>
                <button
                  type="button"
                  className={`mode-button ${creationMode === 'all' ? 'active' : ''}`}
                  onClick={() => setCreationMode('all')}
                >
                  Todas as Conexões
                </button>
              </div>
              {creationMode === 'all' && (
                <p className="mode-description">
                  Este modo criará o template para todas as conexões com App ID válido no banco de dados.
                </p>
              )}
            </div>

            <div className="form-group">
              <h2>Tipo de Template</h2>
              <div className="template-type-selector">
                <button
                  type="button"
                  className={`type-button ${templateType === 'marketing' ? 'active' : ''}`}
                  onClick={() => setTemplateType('marketing')}
                >
                  <div className="icon marketing-icon"></div>
                  Marketing
                </button>
                <button
                  type="button"
                  className={`type-button ${templateType === 'utility' ? 'active' : ''}`}
                  onClick={() => setTemplateType('utility')}
                >
                  <div className="icon utility-icon"></div>
                  Utility
                </button>
              </div>
            </div>

            <div className="form-group">
              <h2>Header</h2>
              <div className="radio-group">
                <label>
                  <input
                    type="radio"
                    name="headerType"
                    checked={headerType === 'none'}
                    onChange={() => setHeaderType('none')}
                  />
                  Sem Header
                </label>
                <label>
                  <input
                    type="radio"
                    name="headerType"
                    checked={headerType === 'text'}
                    onChange={() => setHeaderType('text')}
                  />
                  Header de Texto
                </label>
                <label>
                  <input
                    type="radio"
                    name="headerType"
                    checked={headerType === 'image'}
                    onChange={() => setHeaderType('image')}
                  />
                  Header de Imagem
                </label>
              </div>

              {headerType === 'text' && (
                <div className="header-text-input">
                  <input
                    type="text"
                    placeholder="Enter header text"
                    value={headerText}
                    onChange={(e) => setHeaderText(e.target.value)}
                  />
                </div>
              )}

              {headerType === 'image' && (
                <div className="image-upload">
                  <label htmlFor="header-image" className="image-upload-label">
                    {headerImage ? 'Change Image' : 'Upload Image'}
                  </label>
                  <input
                    id="header-image"
                    type="file"
                    accept="image/*"
                    onChange={handleImageUpload}
                    className="file-input"
                  />
                </div>
              )}
            </div>

            <div className="form-group">
              <h2>Corpo</h2>
              <textarea
                placeholder="Insira sua mensagem do corpo aqui..."
                value={bodyText}
                onChange={(e) => setBodyText(e.target.value)}
                rows={5}
                required
              />
              <p className="variable-hint">
                Use variáveis como &#123;&#123;1&#125;&#125; ou &#123;&#123;2&#125;&#125; para personalizar sua mensagem
                <br />
                Pressione Enter para adicionar quebras de linha
              </p>
            </div>

            <div className="form-group">
              <h2>Configurações da API</h2>
              <div className="api-settings">
                {creationMode === 'single' && (
                  <div className="input-group">
                    <label htmlFor="app-id">App ID</label>
                    <input
                      id="app-id"
                      type="text"
                      value={appId}
                      onChange={(e) => setAppId(e.target.value)}
                      placeholder="Insira o ID da aplicação"
                      required={creationMode === 'single'}
                    />
                  </div>
                )}
                
                <div className="input-group">
                  <label htmlFor="vertical">Vertical</label>
                  <input
                    id="vertical"
                    type="text"
                    value={vertical}
                    onChange={(e) => setVertical(e.target.value)}
                    placeholder="Insira o Vertical"
                    required
                  />
                </div>
              </div>
            </div>

            {bulkResult && (
              <div className="result-summary">
                <h3>Resultado da Criação</h3>
                <p>Templates criados: {bulkResult.successful} de {bulkResult.total}</p>
                <div className="app-ids-list">
                  <p>App IDs processados:</p>
                  <ul>
                    {bulkResult.app_ids.map((id, index) => (
                      <li key={index}>{id}</li>
                    ))}
                  </ul>
                </div>
              </div>
            )}

            <div className="form-actions">
              <button 
                type="submit" 
                className="send-button" 
                disabled={isSubmitting || isProcessing}
              >
                {isSubmitting 
                  ? 'Enviando...' 
                  : creationMode === 'single' 
                    ? 'Salvar Template' 
                    : `Criar para Todas Conexões${isProcessing ? ' (Processando...)' : ''}`
                }
              </button>
            </div>
          </form>
        </div>
        
        <div className="template-creator__preview-section">
          <h2>Preview</h2>
          <div className="template-preview">
            {templateName && (
              <div className="template-name-badge">
                {templateName}
              </div>
            )}
            <div className="phone-mockup">
              <div className="phone-screen">
                <div className="message-bubble">
                  {headerType === 'image' && headerImage && (
                    <div className="message-header-image">
                      <img src={headerImage} alt="Header" />
                    </div>
                  )}
                  
                  {headerType === 'text' && headerText && (
                    <div className="message-header-text">{headerText}</div>
                  )}
                  
                  <div className="message-body">
                    {bodyText ? formatPreviewText(bodyText) : 'Sua mensagem vai aparecer aqui'}
                  </div>
                </div>
              </div>
            </div>
            <div className={`template-type-badge ${templateType}`}>
              {templateType === 'marketing' ? 'Marketing' : 'Utility'}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default TemplateCreator;
