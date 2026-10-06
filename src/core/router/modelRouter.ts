import {
  ModelDescriptor,
  ProviderHealthStatus,
  NormalizedChatRequest,
  NormalizedChatResponse,
  StreamEvent,
} from '../types/gateway.js';
import { AgentRole } from '../types/agent.js';
import { UniversalModelGateway } from '../gateway/universalGateway.js';
import { CircuitBreaker } from './circuitBreaker.js';
import { UIEventEmitter } from '../events/uiEventEmitter.js';
import { ICancellationToken } from '../types/agent.js';
import { CloudConsentGuard } from '../security/cloudConsentGuard.js';

export type RoutingStrategyMode = 
  | 'AUTO' 
  | 'FAST' 
  | 'POWERFUL' 
  | 'FREE_ONLY' 
  | 'LOCAL_ONLY' 
  | 'CUSTOM';

export interface RouteRequest {
  taskId: string;
  workspacePath?: string;
  role?: AgentRole;
  requiresTools?: boolean;
  requiresVision?: boolean;
  requiresReasoning?: boolean;
  minContextWindow?: number;
  strategyMode?: RoutingStrategyMode;
  customProviderId?: string;
  customModelId?: string;
  excludeProviderIds?: string[];
}

export interface RouteSelection {
  providerId: string;
  modelId: string;
  score: number;
  reason: string;
}

export class ModelRouter {
  private gateway: UniversalModelGateway;
  private eventEmitter?: UIEventEmitter;
  private breakers: Map<string, CircuitBreaker> = new Map();
  private defaultStrategy: RoutingStrategyMode = 'AUTO';

  constructor(gateway: UniversalModelGateway, eventEmitter?: UIEventEmitter) {
    this.gateway = gateway;
    this.eventEmitter = eventEmitter;
  }

  public setEventEmitter(emitter: UIEventEmitter): void {
    this.eventEmitter = emitter;
  }

  public setDefaultStrategy(mode: RoutingStrategyMode): void {
    this.defaultStrategy = mode;
  }

  public getCircuitBreaker(providerId: string): CircuitBreaker {
    let cb = this.breakers.get(providerId);
    if (!cb) {
      cb = new CircuitBreaker(providerId);
      this.breakers.set(providerId, cb);
    }
    return cb;
  }

  /**
   * Evaluates registered models and selects the optimal healthy candidate.
   */
  public async selectRoute(req: RouteRequest): Promise<RouteSelection> {
    const mode = req.strategyMode || this.defaultStrategy;
    const allModels = await this.gateway.listAllModels();

    if (mode === 'CUSTOM' && req.customProviderId && req.customModelId) {
      this.eventEmitter?.emit(req.taskId, 'model_fallback_occurred', {
        action: 'model.selected',
        providerId: req.customProviderId,
        modelId: req.customModelId,
        strategy: 'CUSTOM',
      });
      return {
        providerId: req.customProviderId,
        modelId: req.customModelId,
        score: 100,
        reason: 'Selected via manual CUSTOM configuration',
      };
    }

    // Filter candidates matching hard criteria
    const eligible = allModels.filter((m) => {
      if (req.excludeProviderIds?.includes(m.providerId)) return false;

      const cb = this.getCircuitBreaker(m.providerId);
      if (!cb.isAvailable()) return false;
      if (m.health === 'AUTH_ERROR' || m.health === 'OFFLINE' || m.health === 'UNSUPPORTED') return false;

      if (req.requiresTools && !m.capabilities.supportsTools) return false;
      if (req.requiresVision && !m.capabilities.supportsVision) return false;
      if (req.requiresReasoning && !m.capabilities.supportsReasoning) return false;
      if (req.minContextWindow && m.capabilities.contextWindow < req.minContextWindow) return false;

      if (mode === 'FREE_ONLY' && !m.capabilities.isFree && !m.capabilities.isLocal) return false;
      if (mode === 'LOCAL_ONLY' && !m.capabilities.isLocal) return false;

      return true;
    });

    if (eligible.length === 0) {
      // Fallback pool without circuit breaker filter
      const remaining = allModels.filter((m) => !req.excludeProviderIds?.includes(m.providerId));
      const fallback = remaining[0] || allModels[0] || {
        providerId: 'mock-provider',
        modelId: 'fallback-v1',
      };
      return {
        providerId: fallback.providerId,
        modelId: fallback.modelId,
        score: 10,
        reason: 'Emergency fallback: no ideal healthy candidates passed filters',
      };
    }

    // Score candidates based on strategy mode
    let bestCandidate = eligible[0];
    let bestScore = -1;
    let selectionReason = 'Default candidate match';

    for (const candidate of eligible) {
      let score = 50;

      if (candidate.health === 'HEALTHY') score += 20;
      if (candidate.health === 'DEGRADED') score -= 10;

      if (mode === 'FAST') {
        if (candidate.providerId === 'groq') score += 30;
        if (candidate.modelId.includes('flash') || candidate.modelId.includes('instant') || candidate.modelId.includes('8b')) score += 20;
      } else if (mode === 'POWERFUL') {
        if (candidate.capabilities.supportsReasoning) score += 30;
        if (candidate.capabilities.contextWindow >= 128000) score += 15;
        if (candidate.modelId.includes('pro') || candidate.modelId.includes('405b') || candidate.modelId.includes('sonnet') || candidate.modelId.includes('r1') || candidate.modelId.includes('glm')) score += 25;
      } else if (mode === 'AUTO') {
        // AUTO Mode: prefer high capability cloud if available, keep Ollama as local fallback
        if (req.role === 'PLANNER' || req.role === 'REVIEWER') {
          if (candidate.capabilities.supportsReasoning) score += 25;
        }
        if (!candidate.capabilities.isLocal) {
          score += 15;
        }
      }

      if (score > bestScore) {
        bestScore = score;
        bestCandidate = candidate;
        selectionReason = `Selected candidate matching mode ${mode} with score ${score}`;
      }
    }

    this.eventEmitter?.emit(req.taskId, 'model_fallback_occurred', {
      action: 'model.selected',
      providerId: bestCandidate.providerId,
      modelId: bestCandidate.modelId,
      strategy: mode,
      score: bestScore,
    });

    return {
      providerId: bestCandidate.providerId,
      modelId: bestCandidate.modelId,
      score: bestScore,
      reason: selectionReason,
    };
  }

  /**
   * Executes a chat completion with automated circuit-breaking and fallback retry.
   */
  public async executeChatWithFallback(
    req: RouteRequest,
    messages: NormalizedChatRequest['messages'],
    cancellationToken?: ICancellationToken
  ): Promise<NormalizedChatResponse> {
    const route = await this.selectRoute(req);
    const cb = this.getCircuitBreaker(route.providerId);

    try {
      const resp = await this.gateway.chat(
        {
          providerId: route.providerId,
          modelId: route.modelId,
          messages,
        },
        cancellationToken,
        req.workspacePath
      );
      cb.recordSuccess();
      return resp;
    } catch (primaryErr: any) {
      // If error is permission denied (e.g. Cloud consent missing), rethrow immediately without useless retry
      if (primaryErr.code === 'PERMISSION_DENIED') {
        throw primaryErr;
      }

      cb.recordFailure();

      this.eventEmitter?.emit(req.taskId, 'model_fallback_occurred', {
        action: 'fallback.started',
        failedProvider: route.providerId,
        failedModel: route.modelId,
        error: primaryErr.message,
      });

      // Attempt fallback to alternative candidate excluding failed provider
      const fallbackRoute = await this.selectRoute({
        ...req,
        excludeProviderIds: [...(req.excludeProviderIds || []), route.providerId],
      });

      try {
        const fallbackResp = await this.gateway.chat(
          {
            providerId: fallbackRoute.providerId,
            modelId: fallbackRoute.modelId,
            messages,
          },
          cancellationToken,
          req.workspacePath
        );

        this.getCircuitBreaker(fallbackRoute.providerId).recordSuccess();
        this.eventEmitter?.emit(req.taskId, 'model_fallback_occurred', {
          action: 'fallback.completed',
          newProvider: fallbackRoute.providerId,
          newModel: fallbackRoute.modelId,
        });

        return fallbackResp;
      } catch (fallbackErr: any) {
        this.eventEmitter?.emit(req.taskId, 'model_fallback_occurred', {
          action: 'fallback.failed',
          error: fallbackErr.message,
        });
        throw fallbackErr;
      }
    }
  }
}
